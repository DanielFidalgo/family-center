use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSource {
    pub id: Uuid,
    pub google_account_id: Uuid,
    pub calendar_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color_hex: Option<String>,
    pub is_selected: bool,
    pub access_role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSelection {
    pub calendar_source_id: Uuid,
    pub is_selected: bool,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SelectCalendarsBody {
    pub selections: Vec<CalendarSelection>,
}
