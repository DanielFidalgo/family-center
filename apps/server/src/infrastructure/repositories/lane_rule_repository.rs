use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::lane::LaneAssignmentRule;
use crate::domain::repositories::lane_rule_repository::ILaneRuleRepository;

pub struct LaneRuleRepository {
    pool: PgPool,
}

impl LaneRuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ILaneRuleRepository for LaneRuleRepository {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LaneAssignmentRule>> {
        let rules = sqlx::query_as!(
            LaneAssignmentRule,
            "SELECT id, household_id, calendar_source_id, email_pattern, person_id, lane_target, priority, created_at, updated_at FROM family_center.lane_assignment_rules WHERE household_id = $1 ORDER BY priority",
            household_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rules)
    }
}
