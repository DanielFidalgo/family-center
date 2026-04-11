use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::merged_event::MergedEventGroupWithSources;
use crate::domain::dedupe::EventGroup;

#[async_trait]
pub trait IMergedEventRepository: Send + Sync {
    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MergedEventGroupWithSources>>;

    async fn delete_by_household(&self, household_id: Uuid) -> anyhow::Result<()>;

    async fn insert_group(
        &self,
        household_id: Uuid,
        group: &EventGroup,
        person_id: Option<Uuid>,
    ) -> anyhow::Result<()>;
}
