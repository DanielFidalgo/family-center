use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvent {
    pub id: Uuid,
    pub calendar_source_id: Uuid,
    pub google_event_id: String,
    pub ical_uid: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
    pub recurring_event_id: Option<String>,
    pub organizer: Option<String>,
    pub attendees: Option<Vec<String>>,
    // raw_json is excluded from OpenAPI schema — serde_json::Value is not supported by poem_openapi::Object
    #[oai(skip)]
    pub raw_json: Value,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
