use std::sync::Arc;
use poem_openapi::{OpenApi, payload::Json};
use uuid::Uuid;
use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::settings::{Settings, UpdateSettings};
use crate::infrastructure::auth;

pub struct SettingsApi {
    pub context: Arc<dyn IAppContext>,
}

impl SettingsApi {
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

#[OpenApi(tag = "ApiTags::Settings")]
impl SettingsApi {
    /// Get household settings. Creates defaults if none exist.
    #[oai(path = "/settings", method = "get")]
    pub async fn get_settings(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Settings>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let settings = self.context.settings_repository().find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;

        match settings {
            Some(s) => Ok(Json(s)),
            None => {
                let s = self.context.settings_repository().create_default(household_id)
                    .await
                    .map_err(ApiError::from)?;
                Ok(Json(s))
            }
        }
    }

    /// Update household settings.
    #[oai(path = "/settings", method = "patch")]
    pub async fn update_settings(
        &self,
        auth: BearerAuth,
        body: Json<UpdateSettings>,
    ) -> Result<Json<Settings>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let updated = self.context.settings_repository().update(household_id, &body.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(updated))
    }
}
