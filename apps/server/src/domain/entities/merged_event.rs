use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct MergedEventGroup {
    pub id: Uuid,
    pub household_id: Uuid,
    pub canonical_title: String,
    pub canonical_start: DateTime<Utc>,
    pub canonical_end: DateTime<Utc>,
    pub is_all_day: bool,
    pub person_id: Option<Uuid>,
    pub lane_override: Option<Uuid>,
    pub dupe_tier: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct MergedEventSource {
    pub id: Uuid,
    pub merged_event_group_id: Uuid,
    pub source_event_id: Uuid,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

/// Flattened version for poem_openapi::Object (no serde flatten supported).
/// MergedEventGroup fields are inlined alongside sources.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct MergedEventGroupWithSources {
    pub id: Uuid,
    pub household_id: Uuid,
    pub canonical_title: String,
    pub canonical_start: DateTime<Utc>,
    pub canonical_end: DateTime<Utc>,
    pub is_all_day: bool,
    pub person_id: Option<Uuid>,
    pub lane_override: Option<Uuid>,
    pub dupe_tier: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sources: Vec<MergedEventSource>,
}

impl MergedEventGroupWithSources {
    pub fn new(group: MergedEventGroup, sources: Vec<MergedEventSource>) -> Self {
        Self {
            id: group.id,
            household_id: group.household_id,
            canonical_title: group.canonical_title,
            canonical_start: group.canonical_start,
            canonical_end: group.canonical_end,
            is_all_day: group.is_all_day,
            person_id: group.person_id,
            lane_override: group.lane_override,
            dupe_tier: group.dupe_tier,
            created_at: group.created_at,
            updated_at: group.updated_at,
            sources,
        }
    }
}
