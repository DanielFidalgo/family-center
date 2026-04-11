use std::sync::Arc;
use poem_openapi::{OpenApi, payload::Json};
use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::sync::{SyncRunRequest, SyncRunResponse};
use crate::infrastructure::auth;

pub struct SyncApi {
    pub context: Arc<dyn IAppContext>,
}

impl SyncApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, self.context.config().jwt_secret())
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "ApiTags::Sync")]
impl SyncApi {
    /// Trigger a calendar sync run.
    #[oai(path = "/sync/run", method = "post")]
    pub async fn run_sync(
        &self,
        auth: BearerAuth,
        body: Json<SyncRunRequest>,
    ) -> Result<Json<SyncRunResponse>, ApiError> {
        self.verify(&auth)?;
        let response = self.context.sync_service().run_sync(body.0)
            .await
            .map_err(ApiError::from)?;
        Ok(Json(response))
    }
}
