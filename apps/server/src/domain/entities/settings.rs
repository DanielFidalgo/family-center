use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub household_id: Uuid,
    pub default_view: String,
    pub week_starts_monday: bool,
    pub dedupe_mode: String,
    pub display_timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettings {
    pub default_view: Option<String>,
    pub week_starts_monday: Option<bool>,
    pub dedupe_mode: Option<String>,
    pub display_timezone: Option<String>,
}
