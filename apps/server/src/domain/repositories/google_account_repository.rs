use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::google_account::GoogleAccount;

#[async_trait]
pub trait IGoogleAccountRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<GoogleAccount>>;

    async fn upsert_mock(&self, household_id: Uuid) -> anyhow::Result<GoogleAccount>;

    async fn upsert(
        &self,
        household_id: Uuid,
        email: &str,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> anyhow::Result<GoogleAccount>;
}
