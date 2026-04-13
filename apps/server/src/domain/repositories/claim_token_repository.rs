use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::claim_token::ClaimToken;

#[async_trait]
pub trait IClaimTokenRepository: Send + Sync {
    async fn generate(&self, person_id: Uuid) -> anyhow::Result<ClaimToken>;
    async fn find_valid(&self, token: &str) -> anyhow::Result<Option<ClaimToken>>;
    async fn delete_expired(&self) -> anyhow::Result<u64>;
}
