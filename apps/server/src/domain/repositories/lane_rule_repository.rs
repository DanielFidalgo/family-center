use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::lane::LaneAssignmentRule;

#[async_trait]
pub trait ILaneRuleRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LaneAssignmentRule>>;

    async fn create(
        &self,
        household_id: Uuid,
        calendar_source_id: Option<Uuid>,
        email_pattern: Option<String>,
        person_id: Option<Uuid>,
        lane_target: &str,
        priority: i32,
    ) -> anyhow::Result<LaneAssignmentRule>;

    async fn delete_by_id(&self, id: Uuid) -> anyhow::Result<()>;

    async fn delete_by_calendar_source_and_person(
        &self,
        household_id: Uuid,
        calendar_source_id: Uuid,
        person_id: Option<Uuid>,
    ) -> anyhow::Result<u64>;
}
