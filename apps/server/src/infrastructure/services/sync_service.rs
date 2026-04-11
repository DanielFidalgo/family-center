use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::configuration::config::IAppConfig;
use crate::domain::dedupe::{apply_lane_rules, group_events};
use crate::domain::entities::source_event::SourceEvent;
use crate::domain::entities::sync::{SyncError, SyncRunRequest, SyncRunResponse};
use crate::domain::repositories::{
    calendar_source_repository::ICalendarSourceRepository,
    household_repository::IHouseholdRepository,
    lane_rule_repository::ILaneRuleRepository,
    merged_event_repository::IMergedEventRepository,
    source_event_repository::ISourceEventRepository,
};
use crate::domain::sync::ISyncService;

pub struct SyncService {
    config: Arc<dyn IAppConfig>,
    calendar_source_repo: Arc<dyn ICalendarSourceRepository>,
    source_event_repo: Arc<dyn ISourceEventRepository>,
    household_repo: Arc<dyn IHouseholdRepository>,
    merged_event_repo: Arc<dyn IMergedEventRepository>,
    lane_rule_repo: Arc<dyn ILaneRuleRepository>,
}

impl SyncService {
    pub fn new(
        config: Arc<dyn IAppConfig>,
        calendar_source_repo: Arc<dyn ICalendarSourceRepository>,
        source_event_repo: Arc<dyn ISourceEventRepository>,
        household_repo: Arc<dyn IHouseholdRepository>,
        merged_event_repo: Arc<dyn IMergedEventRepository>,
        lane_rule_repo: Arc<dyn ILaneRuleRepository>,
    ) -> Self {
        Self {
            config,
            calendar_source_repo,
            source_event_repo,
            household_repo,
            merged_event_repo,
            lane_rule_repo,
        }
    }

    async fn sync_one_calendar(
        &self,
        calendar_source_id: Uuid,
        access_token: Option<&str>,
        _force_full: bool,
    ) -> anyhow::Result<(u32, u32)> {
        let events = if self.config.mock_calendar() || access_token.is_none() {
            let start = chrono::Utc::now() - chrono::Duration::days(7);
            let end = chrono::Utc::now() + chrono::Duration::days(30);
            crate::infrastructure::google::mock::mock_events_for_range(calendar_source_id, start, end)
        } else {
            let token = access_token.unwrap();
            fetch_google_events(calendar_source_id, token).await?
        };

        let mut created = 0u32;
        let mut updated = 0u32;

        for event in &events {
            let is_new = self.source_event_repo.upsert(event).await?;
            if is_new {
                created += 1;
            } else {
                updated += 1;
            }
        }

        Ok((created, updated))
    }

    async fn rebuild_merged_events(&self) -> anyhow::Result<()> {
        let household_ids = self.household_repo.find_all_ids().await?;

        for household_id in household_ids {
            self.rebuild_for_household(household_id).await?;
        }
        Ok(())
    }

    async fn rebuild_for_household(&self, household_id: Uuid) -> anyhow::Result<()> {
        let events = self.source_event_repo.find_by_household_selected(household_id).await?;
        let rules = self.lane_rule_repo.find_by_household(household_id).await?;

        let groups = group_events(&events);

        self.merged_event_repo.delete_by_household(household_id).await?;

        for group in &groups {
            if group.members.is_empty() {
                continue;
            }

            let primary_id = group.members.iter()
                .find(|(_, p)| *p)
                .map(|(id, _)| *id)
                .unwrap_or(group.members[0].0);

            let person_id = events.iter()
                .find(|e| e.id == primary_id)
                .and_then(|e| apply_lane_rules(e, &rules));

            self.merged_event_repo.insert_group(household_id, group, person_id).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl ISyncService for SyncService {
    async fn run_sync(&self, req: SyncRunRequest) -> anyhow::Result<SyncRunResponse> {
        let mut response = SyncRunResponse {
            synced: 0,
            created: 0,
            updated: 0,
            errors: Vec::new(),
        };

        let sources = if let Some(ids) = &req.calendar_source_ids {
            self.calendar_source_repo.find_selected_with_tokens_by_ids(ids).await?
        } else {
            self.calendar_source_repo.find_selected_with_tokens().await?
        };

        for source in sources {
            let result = self.sync_one_calendar(
                source.id,
                source.access_token.as_deref(),
                req.force_full_sync.unwrap_or(false),
            ).await;
            match result {
                Ok((created, updated)) => {
                    response.synced += 1;
                    response.created += created;
                    response.updated += updated;
                }
                Err(e) => {
                    response.errors.push(SyncError {
                        calendar_source_id: source.id,
                        error: e.to_string(),
                    });
                }
            }
        }

        // After sync, rebuild merged event groups
        if response.errors.is_empty() || response.synced > 0 {
            if let Err(e) = self.rebuild_merged_events().await {
                tracing::warn!("Failed to rebuild merged events: {e}");
            }
        }

        Ok(response)
    }
}

async fn fetch_google_events(
    calendar_source_id: Uuid,
    access_token: &str,
) -> anyhow::Result<Vec<SourceEvent>> {
    let http = reqwest::Client::new();

    let start = chrono::Utc::now() - chrono::Duration::days(7);
    let end = chrono::Utc::now() + chrono::Duration::days(30);

    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&maxResults=250",
        start.to_rfc3339(),
        end.to_rfc3339()
    );

    let resp: serde_json::Value = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    let now = chrono::Utc::now();
    let mut events = Vec::new();

    if let Some(items) = resp["items"].as_array() {
        for item in items {
            let start_str = item["start"]["dateTime"]
                .as_str()
                .or_else(|| item["start"]["date"].as_str())
                .unwrap_or("");
            let end_str = item["end"]["dateTime"]
                .as_str()
                .or_else(|| item["end"]["date"].as_str())
                .unwrap_or("");

            let is_all_day = item["start"]["dateTime"].is_null();

            let start_at = chrono::DateTime::parse_from_rfc3339(start_str)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            let end_at = chrono::DateTime::parse_from_rfc3339(end_str)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or(now);

            let attendees: Option<Vec<String>> = item["attendees"]
                .as_array()
                .map(|arr| arr.iter()
                    .filter_map(|a| a["email"].as_str().map(|s| s.to_string()))
                    .collect());

            events.push(SourceEvent {
                id: Uuid::new_v4(),
                calendar_source_id,
                google_event_id: item["id"].as_str().unwrap_or("").to_string(),
                ical_uid: item["iCalUID"].as_str().map(|s| s.to_string()),
                title: item["summary"].as_str().unwrap_or("(no title)").to_string(),
                description: item["description"].as_str().map(|s| s.to_string()),
                location: item["location"].as_str().map(|s| s.to_string()),
                start_at,
                end_at,
                is_all_day,
                recurrence_rule: item["recurrence"].as_array()
                    .and_then(|r| r.first())
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string()),
                recurring_event_id: item["recurringEventId"].as_str().map(|s| s.to_string()),
                organizer: item["organizer"]["email"].as_str().map(|s| s.to_string()),
                attendees,
                raw_json: item.clone(),
                synced_at: now,
                created_at: now,
                updated_at: now,
            });
        }
    }

    Ok(events)
}
