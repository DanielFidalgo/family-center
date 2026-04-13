use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::entities::local_activity::{
    ActivityCompletion, CreateLocalActivity, LocalActivity, LocalActivityWithRecurrence,
    UpdateLocalActivity,
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

    async fn complete_activity(
        &self,
        activity_id: Uuid,
        date: NaiveDate,
        completed_by: Option<Uuid>,
    ) -> anyhow::Result<ActivityCompletion>;

    async fn uncomplete_activity(
        &self,
        activity_id: Uuid,
        date: NaiveDate,
    ) -> anyhow::Result<()>;

    async fn find_completions_in_range(
        &self,
        household_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<ActivityCompletion>>;
}
