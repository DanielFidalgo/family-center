use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimToken {
    pub id: Uuid,
    pub person_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub claim_url: String,
}
