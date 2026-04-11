use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct LaneAssignmentRule {
    pub id: Uuid,
    pub household_id: Uuid,
    pub calendar_source_id: Option<Uuid>,
    pub email_pattern: Option<String>,
    pub person_id: Option<Uuid>,
    pub lane_target: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
