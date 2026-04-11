use async_trait::async_trait;

use crate::domain::entities::sync::{SyncRunRequest, SyncRunResponse};

#[async_trait]
pub trait ISyncService: Send + Sync {
    async fn run_sync(&self, req: SyncRunRequest) -> anyhow::Result<SyncRunResponse>;
}
