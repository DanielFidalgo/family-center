use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::google_account::GoogleAccount;
use crate::domain::repositories::google_account_repository::IGoogleAccountRepository;

pub struct GoogleAccountRepository {
    pool: PgPool,
}

impl GoogleAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IGoogleAccountRepository for GoogleAccountRepository {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<GoogleAccount>> {
        let accounts = sqlx::query_as!(
            GoogleAccount,
            "SELECT id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active, created_at, updated_at FROM family_center.google_accounts WHERE household_id = $1 AND is_active = TRUE ORDER BY created_at",
            household_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(accounts)
    }

    async fn upsert_mock(&self, household_id: Uuid) -> anyhow::Result<GoogleAccount> {
        let account = sqlx::query_as!(
            GoogleAccount,
            r#"INSERT INTO family_center.google_accounts (id, household_id, email, display_name, is_active)
               VALUES ($1, $2, 'mock@example.com', 'Mock Account', TRUE)
               ON CONFLICT (household_id, email) DO UPDATE SET display_name = EXCLUDED.display_name
               RETURNING id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active, created_at, updated_at"#,
            Uuid::new_v4(),
            household_id,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(account)
    }

    async fn upsert(
        &self,
        household_id: Uuid,
        email: &str,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> anyhow::Result<GoogleAccount> {
        let account = sqlx::query_as!(
            GoogleAccount,
            r#"INSERT INTO family_center.google_accounts (id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
               ON CONFLICT (household_id, email) DO UPDATE SET
                   display_name = EXCLUDED.display_name,
                   avatar_url = EXCLUDED.avatar_url,
                   access_token = EXCLUDED.access_token,
                   refresh_token = COALESCE(EXCLUDED.refresh_token, google_accounts.refresh_token),
                   token_expires_at = EXCLUDED.token_expires_at,
                   updated_at = NOW()
               RETURNING id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active, created_at, updated_at"#,
            Uuid::new_v4(),
            household_id,
            email,
            display_name,
            avatar_url,
            access_token,
            refresh_token,
            token_expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(account)
    }
}
