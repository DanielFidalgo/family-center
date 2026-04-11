use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncCheckpoint {
    pub id: Uuid,
    pub calendar_source_id: Uuid,
    pub sync_token: Option<String>,
    pub full_sync_at: Option<DateTime<Utc>>,
    pub next_page_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunRequest {
    pub calendar_source_ids: Option<Vec<Uuid>>,
    pub force_full_sync: Option<bool>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunResponse {
    pub synced: u32,
    pub created: u32,
    pub updated: u32,
    pub errors: Vec<SyncError>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub calendar_source_id: Uuid,
    pub error: String,
}
