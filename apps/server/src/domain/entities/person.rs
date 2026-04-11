use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: Uuid,
    pub household_id: Uuid,
    pub name: String,
    pub color: String,
    pub avatar_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CreatePerson {
    pub name: String,
    pub color: String,
    pub avatar_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub color: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}
