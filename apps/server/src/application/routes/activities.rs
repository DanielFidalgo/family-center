use std::sync::Arc;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;
use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::local_activity::{
    CreateLocalActivity, LocalActivityWithRecurrence, UpdateLocalActivity,
};
use crate::infrastructure::auth;

pub struct ActivitiesApi {
    pub context: Arc<dyn IAppContext>,
}

impl ActivitiesApi {
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

#[OpenApi(tag = "ApiTags::Activities")]
impl ActivitiesApi {
    /// List all local activities with their recurrence rules.
    #[oai(path = "/activities", method = "get")]
    pub async fn list_activities(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<LocalActivityWithRecurrence>>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let activities = self.context.local_activity_repository().find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(activities))
    }

    /// Create a new local activity, optionally with a recurrence rule.
    #[oai(path = "/activities", method = "post")]
    pub async fn create_activity(
        &self,
        auth: BearerAuth,
        body: Json<CreateLocalActivity>,
    ) -> Result<Json<LocalActivityWithRecurrence>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let activity = self.context.local_activity_repository().create(household_id, body.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(activity))
    }

    /// Update an existing local activity by ID.
    #[oai(path = "/activities/:id", method = "patch")]
    pub async fn update_activity(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
        body: Json<UpdateLocalActivity>,
    ) -> Result<Json<LocalActivityWithRecurrence>, ApiError> {
        self.verify(&auth)?;

        let _existing = self.context.local_activity_repository().find_by_id(id.0)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Activity {} not found", id.0)))?;

        let activity = self.context.local_activity_repository().update(id.0, body.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(activity))
    }
}
