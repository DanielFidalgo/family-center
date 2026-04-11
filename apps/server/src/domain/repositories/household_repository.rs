use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::household::Household;

#[async_trait]
pub trait IHouseholdRepository: Send + Sync {
    async fn find_first(&self) -> anyhow::Result<Option<Household>>;
    async fn create(&self, id: Uuid, name: &str) -> anyhow::Result<Household>;
    async fn find_all_ids(&self) -> anyhow::Result<Vec<Uuid>>;
}
