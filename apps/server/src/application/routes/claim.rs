use std::sync::Arc;

use poem_openapi::{
    Object, OpenApi,
    param::{Path, Query},
    payload::{Binary, Html, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::routes::security::{ApiError, ApiTags};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::claim_token::ClaimToken;
use crate::domain::entities::person::{Person, UpdatePerson};

pub struct ClaimApi {
    pub context: Arc<dyn IAppContext>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimProfileResponse {
    pub person: Person,
    pub linked_google_emails: Vec<String>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClaimProfileRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct OAuthStartResponse {
    pub auth_url: String,
    pub state: String,
}

impl ClaimApi {
    async fn validate_token(&self, token: &str) -> Result<ClaimToken, ApiError> {
        self.context
            .claim_token_repository()
            .find_valid(token)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::gone("Claim token is invalid or has expired"))
    }

    async fn household_id(&self) -> Result<Uuid, ApiError> {
        let h = self
            .context
            .household_repository()
            .find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| {
                ApiError::bad_request("No household configured. Call /auth/bootstrap first.")
            })?;
        Ok(h.id)
    }
}

#[OpenApi(tag = "ApiTags::Claim")]
impl ClaimApi {
    /// Get the profile associated with a claim token.
    #[oai(path = "/claim/:token", method = "get")]
    pub async fn get_profile(
        &self,
        token: Path<String>,
    ) -> Result<Json<ClaimProfileResponse>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        let person = self
            .context
            .person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", claim.person_id)))?;

        let household_id = person.household_id;

        // Find lane rules for this person
        let all_rules = self
            .context
            .lane_rule_repository()
            .find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;

        let person_has_rules = all_rules
            .iter()
            .any(|r| r.person_id == Some(claim.person_id));

        let linked_google_emails = if person_has_rules {
            // Get all household Google accounts and return their emails
            let accounts = self
                .context
                .google_account_repository()
                .find_by_household(household_id)
                .await
                .map_err(ApiError::from)?;
            accounts.into_iter().map(|a| a.email).collect()
        } else {
            vec![]
        };

        Ok(Json(ClaimProfileResponse {
            person,
            linked_google_emails,
        }))
    }

    /// Update the profile associated with a claim token.
    #[oai(path = "/claim/:token", method = "patch")]
    pub async fn update_profile(
        &self,
        token: Path<String>,
        body: Json<UpdateClaimProfileRequest>,
    ) -> Result<Json<Person>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        let existing = self
            .context
            .person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", claim.person_id)))?;

        let update = UpdatePerson {
            name: body.0.name,
            color: body.0.color,
            avatar_url: None,
            sort_order: None,
            is_active: None,
        };

        let person = self
            .context
            .person_repository()
            .update(claim.person_id, &existing, &update)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(person))
    }

    /// Upload an avatar for the person associated with a claim token.
    #[oai(path = "/claim/:token/avatar", method = "post")]
    pub async fn upload_avatar(
        &self,
        token: Path<String>,
        content_type: Query<String>,
        body: Binary<Vec<u8>>,
    ) -> Result<Json<Person>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        // Validate max 5 MB
        if body.0.len() > 5 * 1024 * 1024 {
            return Err(ApiError::bad_request("Avatar must be 5 MB or smaller"));
        }

        let existing = self
            .context
            .person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", claim.person_id)))?;

        let avatar_url = crate::infrastructure::s3::upload_avatar(
            self.context.config(),
            &claim.person_id.to_string(),
            &content_type.0,
            &body.0,
        )
        .await
        .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        let update = UpdatePerson {
            name: None,
            color: None,
            avatar_url: Some(Some(avatar_url)),
            sort_order: None,
            is_active: None,
        };

        let person = self
            .context
            .person_repository()
            .update(claim.person_id, &existing, &update)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(person))
    }

    /// Start the Google OAuth flow for the person associated with a claim token.
    #[oai(path = "/claim/:token/google/start", method = "post")]
    pub async fn google_start(
        &self,
        token: Path<String>,
    ) -> Result<Json<OAuthStartResponse>, ApiError> {
        // Validate the claim token exists
        let _claim = self.validate_token(&token.0).await?;

        if self.context.config().mock_calendar() {
            let mock_url = format!(
                "/api/claim/google/callback?code=mock&state={}",
                token.0
            );
            return Ok(Json(OAuthStartResponse {
                auth_url: mock_url,
                state: token.0.clone(),
            }));
        }

        let public_url = self.context.config().public_url();
        let redirect_uri = format!("{}/api/claim/google/callback", public_url);

        let mut client =
            crate::infrastructure::google::client::build_oauth_client_from_config(
                self.context.config(),
            )
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        // Override the redirect URI to the claim-specific callback
        use oauth2::RedirectUrl;
        client = client.set_redirect_uri(
            RedirectUrl::new(redirect_uri).map_err(|e| ApiError::bad_request(e.to_string()))?,
        );

        use oauth2::{CsrfToken, Scope};
        // Encode the claim token as the CSRF state
        let (auth_url, csrf_token) = client
            .authorize_url(|| CsrfToken::new(token.0.clone()))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/calendar.readonly".to_string(),
            ))
            .url();

        Ok(Json(OAuthStartResponse {
            auth_url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        }))
    }

    /// Google OAuth callback for claim flow — exchanges code and links calendars to person.
    #[oai(path = "/claim/google/callback", method = "get")]
    pub async fn google_callback(
        &self,
        code: Query<Option<String>>,
        state: Query<Option<String>>,
        error: Query<Option<String>>,
    ) -> Result<Html<String>, ApiError> {
        if let Some(err) = error.0 {
            return Err(ApiError::bad_request(format!("OAuth error: {err}")));
        }

        // The state param carries the claim token
        let claim_token_str = state
            .0
            .ok_or_else(|| ApiError::bad_request("Missing state parameter"))?;

        let claim = self.validate_token(&claim_token_str).await?;
        let household_id = self.household_id().await?;

        let person = self
            .context
            .person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", claim.person_id)))?;

        let google_account = if self.context.config().mock_calendar()
            || code.0.as_deref() == Some("mock")
        {
            let account = self
                .context
                .google_account_repository()
                .upsert_mock(household_id)
                .await
                .map_err(ApiError::from)?;

            self.context
                .calendar_source_repository()
                .create_mock_calendars(account.id)
                .await
                .map_err(ApiError::from)?;

            account
        } else {
            let code_str = code
                .0
                .ok_or_else(|| ApiError::bad_request("Missing code parameter"))?;

            // Build client with the claim-specific redirect URI
            let public_url = self.context.config().public_url().to_string();
            let redirect_uri = format!("{}/api/claim/google/callback", public_url);

            // Temporarily override redirect URI for token exchange
            // We do this by building a fresh config-derived client and resetting the redirect
            let mut client =
                crate::infrastructure::google::client::build_oauth_client_from_config(
                    self.context.config(),
                )
                .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

            use oauth2::RedirectUrl;
            client = client.set_redirect_uri(
                RedirectUrl::new(redirect_uri)
                    .map_err(|e| ApiError::bad_request(e.to_string()))?,
            );

            use oauth2::{AuthorizationCode, TokenResponse};
            let token = client
                .exchange_code(AuthorizationCode::new(code_str))
                .request_async(oauth2::reqwest::async_http_client)
                .await
                .map_err(|e| ApiError::from(anyhow::Error::msg(e.to_string())))?;

            let access_token = token.access_token().secret().clone();
            let refresh_token = token.refresh_token().map(|t| t.secret().clone());
            let expires_in = token.expires_in().map(|d| {
                chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64)
            });

            let http = reqwest::Client::new();
            let user_info: serde_json::Value = http
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(&access_token)
                .send()
                .await
                .map_err(|e| ApiError::from(anyhow::Error::from(e)))?
                .json()
                .await
                .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

            let email = user_info["email"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let display_name = user_info["name"].as_str().map(|s| s.to_string());
            let avatar_url = user_info["picture"].as_str().map(|s| s.to_string());

            self.context
                .google_account_repository()
                .upsert(
                    household_id,
                    &email,
                    display_name.as_deref(),
                    avatar_url.as_deref(),
                    Some(&access_token),
                    refresh_token.as_deref(),
                    expires_in,
                )
                .await
                .map_err(ApiError::from)?
        };

        // Auto-link calendars for this person via lane rules
        let calendars = self
            .context
            .calendar_source_repository()
            .find_by_account(google_account.id)
            .await
            .map_err(ApiError::from)?;

        let lane_target = "person";
        for cal in &calendars {
            self.context
                .lane_rule_repository()
                .create(
                    person.household_id,
                    Some(cal.id),
                    None,
                    Some(claim.person_id),
                    lane_target,
                    100,
                )
                .await
                .map_err(ApiError::from)?;
        }

        let public_url = self.context.config().public_url();
        let redirect_url = format!("{}/claim/{}?google=success", public_url, claim_token_str);

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Redirecting...</title>
<meta http-equiv="refresh" content="0; url={redirect_url}">
</head>
<body>
<p>Redirecting... <a href="{redirect_url}">Click here if not redirected</a></p>
</body>
</html>"#
        );

        Ok(Html(html))
    }
}
