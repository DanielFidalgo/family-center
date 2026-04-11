use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::settings::{Settings, UpdateSettings};

#[async_trait]
pub trait ISettingsRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Option<Settings>>;
    async fn create_default(&self, household_id: Uuid) -> anyhow::Result<Settings>;
    async fn update(&self, household_id: Uuid, input: &UpdateSettings) -> anyhow::Result<Settings>;
}
