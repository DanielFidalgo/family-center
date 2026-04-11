use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::dedupe::EventGroup;
use crate::domain::entities::merged_event::{
    MergedEventGroup, MergedEventGroupWithSources, MergedEventSource,
};
use crate::domain::repositories::merged_event_repository::IMergedEventRepository;

pub struct MergedEventRepository {
    pool: PgPool,
}

impl MergedEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IMergedEventRepository for MergedEventRepository {
    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MergedEventGroupWithSources>> {
        let groups = sqlx::query_as!(
            MergedEventGroup,
            r#"SELECT id, household_id, canonical_title, canonical_start, canonical_end,
                      is_all_day, person_id, lane_override, dupe_tier, created_at, updated_at
               FROM family_center.merged_event_groups
               WHERE household_id = $1 AND canonical_start >= $2 AND canonical_start < $3
               ORDER BY canonical_start"#,
            household_id,
            start,
            end,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(groups.len());
        for group in groups {
            let sources = sqlx::query_as!(
                MergedEventSource,
                "SELECT id, merged_event_group_id, source_event_id, is_primary, created_at FROM family_center.merged_event_sources WHERE merged_event_group_id = $1",
                group.id
            )
            .fetch_all(&self.pool)
            .await?;
            events.push(MergedEventGroupWithSources::new(group, sources));
        }

        Ok(events)
    }

    async fn delete_by_household(&self, household_id: Uuid) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM family_center.merged_event_groups WHERE household_id = $1", household_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_group(
        &self,
        household_id: Uuid,
        group: &EventGroup,
        person_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        if group.members.is_empty() {
            return Ok(());
        }

        let group_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO family_center.merged_event_groups
               (id, household_id, canonical_title, canonical_start, canonical_end, is_all_day, person_id, dupe_tier)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            group_id,
            household_id,
            group.canonical_title,
            group.canonical_start,
            group.canonical_end,
            group.is_all_day,
            person_id,
            group.dupe_tier.as_ref().map(|t| t.as_str()),
        )
        .execute(&self.pool)
        .await?;

        for (source_event_id, is_primary) in &group.members {
            sqlx::query!(
                "INSERT INTO family_center.merged_event_sources (id, merged_event_group_id, source_event_id, is_primary) VALUES ($1, $2, $3, $4)",
                Uuid::new_v4(),
                group_id,
                source_event_id,
                is_primary,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
