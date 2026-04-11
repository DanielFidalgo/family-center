use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};

#[async_trait]
pub trait IPersonRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<Person>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Person>>;
    async fn create(&self, household_id: Uuid, input: &CreatePerson) -> anyhow::Result<Person>;
    async fn update(&self, id: Uuid, existing: &Person, input: &UpdatePerson) -> anyhow::Result<Person>;
}
