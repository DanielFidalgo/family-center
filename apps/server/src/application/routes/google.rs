use std::sync::Arc;
use poem_openapi::{OpenApi, Object, param::{Path, Query}, payload::Json};
use serde::Serialize;
use uuid::Uuid;
use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::{
    calendar_source::{CalendarSource, SelectCalendarsBody},
    google_account::GoogleAccount,
};
use crate::infrastructure::auth;

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ConnectStartResponse {
    pub auth_url: String,
    pub state: String,
}

pub struct GoogleApi {
    pub context: Arc<dyn IAppContext>,
}

impl GoogleApi {
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

#[OpenApi(tag = "ApiTags::Google")]
impl GoogleApi {
    /// Start the Google OAuth2 flow. Returns an auth URL to redirect the user to.
    #[oai(path = "/google/connect/start", method = "post")]
    pub async fn connect_start(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<ConnectStartResponse>, ApiError> {
        self.verify(&auth)?;

        if self.context.config().mock_calendar() {
            return Ok(Json(ConnectStartResponse {
                auth_url: "/google/connect/callback?code=mock&state=mock".to_string(),
                state: "mock".to_string(),
            }));
        }

        let client = crate::infrastructure::google::client::build_oauth_client_from_config(self.context.config())
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;
        use oauth2::{CsrfToken, Scope};
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
            .url();

        Ok(Json(ConnectStartResponse {
            auth_url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        }))
    }

    /// OAuth2 callback — Google redirects here with an authorization code.
    #[oai(path = "/google/connect/callback", method = "get")]
    pub async fn connect_callback(
        &self,
        code: Query<Option<String>>,
        state: Query<Option<String>>,
        error: Query<Option<String>>,
    ) -> Result<Json<GoogleAccount>, ApiError> {
        let _ = state; // unused but part of OAuth protocol
        if let Some(err) = error.0 {
            return Err(ApiError::bad_request(format!("OAuth error: {err}")));
        }

        let household_id = self.household_id().await?;

        if self.context.config().mock_calendar() || code.0.as_deref() == Some("mock") {
            let account = self.context.google_account_repository().upsert_mock(household_id)
                .await
                .map_err(ApiError::from)?;

            self.context.calendar_source_repository().create_mock_calendars(account.id)
                .await
                .map_err(ApiError::from)?;

            return Ok(Json(account));
        }

        let code_str = code.0.ok_or_else(|| ApiError::bad_request("Missing code"))?;
        let account = crate::infrastructure::google::client::exchange_code_and_persist(
            self.context.config(),
            &code_str,
            household_id,
            self.context.google_account_repository(),
        )
        .await
        .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        Ok(Json(account))
    }

    /// List all connected Google accounts.
    #[oai(path = "/google/accounts", method = "get")]
    pub async fn list_accounts(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<GoogleAccount>>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let accounts = self.context.google_account_repository().find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(accounts))
    }

    /// List calendars for a specific Google account.
    #[oai(path = "/google/accounts/:id/calendars", method = "get")]
    pub async fn list_calendars(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
    ) -> Result<Json<Vec<CalendarSource>>, ApiError> {
        self.verify(&auth)?;

        let calendars = self.context.calendar_source_repository().find_by_account(id.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(calendars))
    }

    /// Toggle calendar selection for sync.
    #[oai(path = "/google/calendars/select", method = "post")]
    pub async fn select_calendars(
        &self,
        auth: BearerAuth,
        body: Json<SelectCalendarsBody>,
    ) -> Result<Json<Vec<CalendarSource>>, ApiError> {
        self.verify(&auth)?;
        let mut updated = Vec::new();

        for sel in &body.0.selections {
            let cal = self.context.calendar_source_repository().update_selection(sel)
                .await
                .map_err(ApiError::from)?;

            if let Some(c) = cal {
                updated.push(c);
            }
        }

        Ok(Json(updated))
    }
}
