use std::sync::Arc;
use poem_openapi::{OpenApi, Object, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::configuration::app_context::IAppContext;
use crate::infrastructure::auth;
use crate::application::routes::security::{ApiError, ApiTags};

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapRequest {
    pub household_name: Option<String>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub household_id: Uuid,
    pub token: String,
    pub is_new: bool,
}

pub struct AuthApi {
    pub context: Arc<dyn IAppContext>,
}

#[OpenApi(tag = "ApiTags::Auth")]
impl AuthApi {
    /// Bootstrap the household. Returns a JWT token.
    /// Creates a new household on first call; returns the existing one thereafter.
    #[oai(path = "/auth/bootstrap", method = "post")]
    pub async fn bootstrap(
        &self,
        body: Json<BootstrapRequest>,
    ) -> Result<Json<BootstrapResponse>, ApiError> {
        let existing = self.context.household_repository().find_first()
            .await
            .map_err(ApiError::from)?;

        let (household, is_new) = if let Some(h) = existing {
            (h, false)
        } else {
            let name = body.0.household_name.clone().unwrap_or_else(|| "My Family".to_string());
            let household = self.context.household_repository().create(Uuid::new_v4(), &name)
                .await
                .map_err(ApiError::from)?;

            self.context.settings_repository().create_default(household.id)
                .await
                .map_err(ApiError::from)?;

            (household, true)
        };

        let token = auth::create_token(household.id, self.context.config().jwt_secret())
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        Ok(Json(BootstrapResponse {
            household_id: household.id,
            token,
            is_new,
        }))
    }
}
