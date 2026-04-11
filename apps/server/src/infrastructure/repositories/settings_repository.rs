use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::settings::{Settings, UpdateSettings};
use crate::domain::repositories::settings_repository::ISettingsRepository;

pub struct SettingsRepository {
    pool: PgPool,
}

impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ISettingsRepository for SettingsRepository {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Option<Settings>> {
        let settings = sqlx::query_as!(
            Settings,
            "SELECT household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at FROM family_center.settings WHERE household_id = $1",
            household_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(settings)
    }

    async fn create_default(&self, household_id: Uuid) -> anyhow::Result<Settings> {
        let settings = sqlx::query_as!(
            Settings,
            "INSERT INTO family_center.settings (household_id) VALUES ($1) ON CONFLICT DO NOTHING RETURNING household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at",
            household_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(settings)
    }

    async fn update(&self, household_id: Uuid, input: &UpdateSettings) -> anyhow::Result<Settings> {
        // Fetch current to merge with partial update
        let current = self.find_by_household(household_id).await?
            .unwrap_or_else(|| Settings {
                household_id,
                default_view: "week".to_string(),
                week_starts_monday: false,
                dedupe_mode: "probable".to_string(),
                display_timezone: "UTC".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });

        let updated = sqlx::query_as!(
            Settings,
            r#"UPDATE family_center.settings SET
                default_view = $2,
                week_starts_monday = $3,
                dedupe_mode = $4,
                display_timezone = $5,
                updated_at = NOW()
               WHERE household_id = $1
               RETURNING household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at"#,
            household_id,
            input.default_view.as_deref().unwrap_or(&current.default_view),
            input.week_starts_monday.unwrap_or(current.week_starts_monday),
            input.dedupe_mode.as_deref().unwrap_or(&current.dedupe_mode),
            input.display_timezone.as_deref().unwrap_or(&current.display_timezone),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(updated)
    }
}
