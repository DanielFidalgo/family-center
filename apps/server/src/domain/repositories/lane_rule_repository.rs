use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::lane::LaneAssignmentRule;

#[async_trait]
pub trait ILaneRuleRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LaneAssignmentRule>>;
}
