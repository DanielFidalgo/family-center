use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::RngCore;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entities::claim_token::ClaimToken;
use crate::domain::repositories::claim_token_repository::IClaimTokenRepository;

pub struct ClaimTokenRepository {
    pool: PgPool,
}

impl ClaimTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IClaimTokenRepository for ClaimTokenRepository {
    async fn generate(&self, person_id: Uuid) -> anyhow::Result<ClaimToken> {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let expires_at = Utc::now() + Duration::minutes(30);

        let row = sqlx::query(
            r#"INSERT INTO family_center.person_claim_tokens (person_id, token, expires_at)
               VALUES ($1, $2, $3)
               ON CONFLICT (person_id) DO UPDATE SET
                   token = EXCLUDED.token,
                   expires_at = EXCLUDED.expires_at,
                   created_at = NOW()
               RETURNING id, person_id, token, expires_at, created_at"#,
        )
        .bind(person_id)
        .bind(&token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(ClaimToken {
            id: row.get("id"),
            person_id: row.get("person_id"),
            token: row.get("token"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        })
    }

    async fn find_valid(&self, token: &str) -> anyhow::Result<Option<ClaimToken>> {
        let row = sqlx::query(
            r#"SELECT id, person_id, token, expires_at, created_at
               FROM family_center.person_claim_tokens
               WHERE token = $1 AND expires_at > NOW()"#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ClaimToken {
            id: r.get("id"),
            person_id: r.get("person_id"),
            token: r.get("token"),
            expires_at: r.get("expires_at"),
            created_at: r.get("created_at"),
        }))
    }

    async fn delete_expired(&self) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "DELETE FROM family_center.person_claim_tokens WHERE expires_at <= NOW()",
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
