use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::source_event::SourceEvent;
use crate::domain::repositories::source_event_repository::ISourceEventRepository;

pub struct SourceEventRepository {
    pool: PgPool,
}

impl SourceEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ISourceEventRepository for SourceEventRepository {
    async fn upsert(&self, event: &SourceEvent) -> anyhow::Result<bool> {
        let result = sqlx::query!(
            r#"INSERT INTO family_center.source_events
               (id, calendar_source_id, google_event_id, ical_uid, title, description, location,
                start_at, end_at, is_all_day, recurrence_rule, recurring_event_id, organizer, attendees, raw_json, synced_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW())
               ON CONFLICT (calendar_source_id, google_event_id) DO UPDATE SET
                   title = EXCLUDED.title,
                   description = EXCLUDED.description,
                   location = EXCLUDED.location,
                   start_at = EXCLUDED.start_at,
                   end_at = EXCLUDED.end_at,
                   is_all_day = EXCLUDED.is_all_day,
                   recurrence_rule = EXCLUDED.recurrence_rule,
                   organizer = EXCLUDED.organizer,
                   attendees = EXCLUDED.attendees,
                   raw_json = EXCLUDED.raw_json,
                   synced_at = NOW(),
                   updated_at = NOW()"#,
            event.id,
            event.calendar_source_id,
            event.google_event_id,
            event.ical_uid,
            event.title,
            event.description,
            event.location,
            event.start_at,
            event.end_at,
            event.is_all_day,
            event.recurrence_rule,
            event.recurring_event_id,
            event.organizer,
            event.attendees.as_deref(),
            event.raw_json,
        )
        .execute(&self.pool)
        .await?;

        // rows_affected() == 1 means insert (new row)
        Ok(result.rows_affected() == 1)
    }

    async fn find_by_household_selected(&self, household_id: Uuid) -> anyhow::Result<Vec<SourceEvent>> {
        let events = sqlx::query_as!(
            SourceEvent,
            r#"SELECT se.id, se.calendar_source_id, se.google_event_id, se.ical_uid,
                      se.title, se.description, se.location,
                      se.start_at, se.end_at, se.is_all_day,
                      se.recurrence_rule, se.recurring_event_id,
                      se.organizer, se.attendees, se.raw_json,
                      se.synced_at, se.created_at, se.updated_at
               FROM family_center.source_events se
               JOIN family_center.calendar_sources cs ON se.calendar_source_id = cs.id
               JOIN family_center.google_accounts ga ON cs.google_account_id = ga.id
               WHERE ga.household_id = $1 AND cs.is_selected = TRUE"#,
            household_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(events)
    }
}
