use chrono::{DateTime, NaiveDate, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivity {
    pub id: Uuid,
    pub household_id: Uuid,
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: bool,
    pub category: Option<String>,
    pub is_time_bound: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivityRecurrence {
    pub id: Uuid,
    pub local_activity_id: Uuid,
    pub freq: String,
    pub interval_val: i32,
    pub by_day_of_week: Option<Vec<i32>>,
    pub by_day_of_month: Option<Vec<i32>>,
    pub until: Option<NaiveDate>,
    pub count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ActivityCompletion {
    pub id: Uuid,
    pub local_activity_id: Uuid,
    pub completed_date: NaiveDate,
    pub completed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Flattened version for poem_openapi::Object (no serde flatten supported).
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivityWithRecurrence {
    pub id: Uuid,
    pub household_id: Uuid,
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: bool,
    pub category: Option<String>,
    pub is_time_bound: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub recurrence: Option<LocalActivityRecurrence>,
}

impl LocalActivityWithRecurrence {
    pub fn new(activity: LocalActivity, recurrence: Option<LocalActivityRecurrence>) -> Self {
        Self {
            id: activity.id,
            household_id: activity.household_id,
            person_id: activity.person_id,
            title: activity.title,
            description: activity.description,
            color: activity.color,
            start_at: activity.start_at,
            end_at: activity.end_at,
            is_all_day: activity.is_all_day,
            category: activity.category,
            is_time_bound: activity.is_time_bound,
            created_at: activity.created_at,
            updated_at: activity.updated_at,
            recurrence,
        }
    }
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct RecurrenceInput {
    pub freq: String,
    pub interval: Option<i32>,
    pub by_day_of_week: Option<Vec<i32>>,
    pub by_day_of_month: Option<Vec<i32>>,
    pub until: Option<String>,
    pub count: Option<i32>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalActivity {
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: Option<bool>,
    pub category: Option<String>,
    pub is_time_bound: Option<bool>,
    pub recurrence: Option<RecurrenceInput>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalActivity {
    pub person_id: Option<Option<Uuid>>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub start_at: Option<Option<DateTime<Utc>>>,
    pub end_at: Option<Option<DateTime<Utc>>>,
    pub is_all_day: Option<bool>,
    pub category: Option<Option<String>>,
    pub is_time_bound: Option<bool>,
    pub recurrence: Option<Option<RecurrenceInput>>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CompleteActivityRequest {
    pub date: String,
    pub completed_by: Option<Uuid>,
}
