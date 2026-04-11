use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};
use crate::domain::repositories::person_repository::IPersonRepository;

pub struct PersonRepository {
    pool: PgPool,
}

impl PersonRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IPersonRepository for PersonRepository {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<Person>> {
        let people = sqlx::query_as!(
            Person,
            r#"SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at
               FROM family_center.people WHERE household_id = $1 AND is_active = TRUE ORDER BY sort_order, name"#,
            household_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(people)
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Person>> {
        let person = sqlx::query_as!(
            Person,
            "SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at FROM family_center.people WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(person)
    }

    async fn create(&self, household_id: Uuid, input: &CreatePerson) -> anyhow::Result<Person> {
        let person = sqlx::query_as!(
            Person,
            r#"INSERT INTO family_center.people (id, household_id, name, color, avatar_url, sort_order)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            Uuid::new_v4(),
            household_id,
            input.name,
            input.color,
            input.avatar_url,
            input.sort_order.unwrap_or(0)
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(person)
    }

    async fn update(&self, id: Uuid, existing: &Person, input: &UpdatePerson) -> anyhow::Result<Person> {
        let person = sqlx::query_as!(
            Person,
            r#"UPDATE family_center.people SET
                name = $2,
                color = $3,
                avatar_url = $4,
                sort_order = $5,
                is_active = $6,
                updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            id,
            input.name.as_deref().unwrap_or(&existing.name),
            input.color.as_deref().unwrap_or(&existing.color),
            input.avatar_url.as_ref().map(|o| o.as_deref()).unwrap_or(existing.avatar_url.as_deref()),
            input.sort_order.unwrap_or(existing.sort_order),
            input.is_active.unwrap_or(existing.is_active),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(person)
    }
}
