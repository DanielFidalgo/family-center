use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::household::Household;
use crate::domain::repositories::household_repository::IHouseholdRepository;

pub struct HouseholdRepository {
    pool: PgPool,
}

impl HouseholdRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IHouseholdRepository for HouseholdRepository {
    async fn find_first(&self) -> anyhow::Result<Option<Household>> {
        let row = sqlx::query_as!(
            Household,
            "SELECT id, name, created_at, updated_at FROM family_center.households LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(&self, id: Uuid, name: &str) -> anyhow::Result<Household> {
        let household = sqlx::query_as!(
            Household,
            "INSERT INTO family_center.households (id, name) VALUES ($1, $2) RETURNING id, name, created_at, updated_at",
            id,
            name
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(household)
    }

    async fn find_all_ids(&self) -> anyhow::Result<Vec<Uuid>> {
        let rows = sqlx::query!("SELECT id FROM family_center.households")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}
