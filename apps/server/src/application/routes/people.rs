use std::sync::Arc;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;
use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};
use crate::infrastructure::auth;

pub struct PeopleApi {
    pub context: Arc<dyn IAppContext>,
}

impl PeopleApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, self.context.config().jwt_secret())
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }

    async fn household_id(&self) -> Result<Uuid, ApiError> {
        let h = self.context.household_repository().find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::bad_request("No household configured. Call /auth/bootstrap first."))?;
        Ok(h.id)
    }
}

#[OpenApi(tag = "ApiTags::People")]
impl PeopleApi {
    /// List all active people in the household.
    #[oai(path = "/people", method = "get")]
    pub async fn list_people(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<Person>>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let people = self.context.person_repository().find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(people))
    }

    /// Create a new person in the household.
    #[oai(path = "/people", method = "post")]
    pub async fn create_person(
        &self,
        auth: BearerAuth,
        body: Json<CreatePerson>,
    ) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let person = self.context.person_repository().create(household_id, &body.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(person))
    }

    /// Update an existing person by ID.
    #[oai(path = "/people/:id", method = "patch")]
    pub async fn update_person(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
        body: Json<UpdatePerson>,
    ) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;

        let existing = self.context.person_repository().find_by_id(id.0)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", id.0)))?;

        let person = self.context.person_repository().update(id.0, &existing, &body.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(person))
    }
}
