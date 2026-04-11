use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::local_activity::{
    CreateLocalActivity, LocalActivity, LocalActivityWithRecurrence, UpdateLocalActivity,
};

#[async_trait]
pub trait ILocalActivityRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LocalActivityWithRecurrence>>;
    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<LocalActivityWithRecurrence>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<LocalActivity>>;
    async fn create(&self, household_id: Uuid, input: CreateLocalActivity) -> anyhow::Result<LocalActivityWithRecurrence>;
    async fn update(&self, id: Uuid, input: UpdateLocalActivity) -> anyhow::Result<LocalActivityWithRecurrence>;
}
