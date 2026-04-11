use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::source_event::SourceEvent;

#[async_trait]
pub trait ISourceEventRepository: Send + Sync {
    /// Upsert a source event. Returns true if the row was newly inserted.
    async fn upsert(&self, event: &SourceEvent) -> anyhow::Result<bool>;
    async fn find_by_household_selected(&self, household_id: Uuid) -> anyhow::Result<Vec<SourceEvent>>;
}
