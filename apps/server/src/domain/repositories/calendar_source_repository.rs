use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::calendar_source::{CalendarSelection, CalendarSource};

/// Joined row: calendar source + google account tokens.
#[derive(Debug, Clone)]
pub struct CalendarSourceWithToken {
    pub id: Uuid,
    pub google_account_id: Uuid,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[async_trait]
pub trait ICalendarSourceRepository: Send + Sync {
    async fn find_by_account(&self, google_account_id: Uuid) -> anyhow::Result<Vec<CalendarSource>>;
    async fn create_mock_calendars(&self, google_account_id: Uuid) -> anyhow::Result<()>;
    async fn update_selection(&self, sel: &CalendarSelection) -> anyhow::Result<Option<CalendarSource>>;
    async fn find_selected_with_tokens(&self) -> anyhow::Result<Vec<CalendarSourceWithToken>>;
    async fn find_selected_with_tokens_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<CalendarSourceWithToken>>;
}
