use async_trait::async_trait;
use sqlx::{PgPool, Row};
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

    async fn create(
        &self,
        household_id: Uuid,
        calendar_source_id: Option<Uuid>,
        email_pattern: Option<String>,
        person_id: Option<Uuid>,
        lane_target: &str,
        priority: i32,
    ) -> anyhow::Result<LaneAssignmentRule> {
        let row = sqlx::query(
            r#"INSERT INTO family_center.lane_assignment_rules
               (household_id, calendar_source_id, email_pattern, person_id, lane_target, priority)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, household_id, calendar_source_id, email_pattern, person_id, lane_target, priority, created_at, updated_at"#,
        )
        .bind(household_id)
        .bind(calendar_source_id)
        .bind(email_pattern)
        .bind(person_id)
        .bind(lane_target)
        .bind(priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(LaneAssignmentRule {
            id: row.get("id"),
            household_id: row.get("household_id"),
            calendar_source_id: row.get("calendar_source_id"),
            email_pattern: row.get("email_pattern"),
            person_id: row.get("person_id"),
            lane_target: row.get("lane_target"),
            priority: row.get("priority"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn delete_by_id(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM family_center.lane_assignment_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_calendar_source_and_person(
        &self,
        household_id: Uuid,
        calendar_source_id: Uuid,
        person_id: Option<Uuid>,
    ) -> anyhow::Result<u64> {
        let result = match person_id {
            Some(pid) => {
                sqlx::query(
                    "DELETE FROM family_center.lane_assignment_rules WHERE household_id = $1 AND calendar_source_id = $2 AND person_id = $3",
                )
                .bind(household_id)
                .bind(calendar_source_id)
                .bind(pid)
                .execute(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "DELETE FROM family_center.lane_assignment_rules WHERE household_id = $1 AND calendar_source_id = $2 AND person_id IS NULL",
                )
                .bind(household_id)
                .bind(calendar_source_id)
                .execute(&self.pool)
                .await?
            }
        };
        Ok(result.rows_affected())
    }
}
