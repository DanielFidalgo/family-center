use anyhow::Result;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use uuid::Uuid;
use crate::configuration::config::IAppConfig;
use crate::domain::entities::google_account::GoogleAccount;
use crate::domain::repositories::google_account_repository::IGoogleAccountRepository;

pub fn build_oauth_client_from_config(config: &dyn IAppConfig) -> Result<BasicClient> {
    let client = BasicClient::new(
        ClientId::new(config.google_client_id().to_string()),
        Some(ClientSecret::new(config.google_client_secret().to_string())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(config.google_redirect_uri().to_string())?);
    Ok(client)
}

pub async fn exchange_code_and_persist(
    config: &dyn IAppConfig,
    code: &str,
    household_id: Uuid,
    google_account_repo: &dyn IGoogleAccountRepository,
) -> Result<GoogleAccount> {
    use oauth2::{AuthorizationCode, TokenResponse};

    let client = build_oauth_client_from_config(config)?;

    let token = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let access_token = token.access_token().secret().clone();
    let refresh_token = token.refresh_token().map(|t| t.secret().clone());
    let expires_in = token.expires_in().map(|d| {
        chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64)
    });

    // Fetch user info
    let http = reqwest::Client::new();
    let user_info: serde_json::Value = http
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await?
        .json()
        .await?;

    let email = user_info["email"].as_str().unwrap_or("unknown").to_string();
    let display_name = user_info["name"].as_str().map(|s| s.to_string());
    let avatar_url = user_info["picture"].as_str().map(|s| s.to_string());

    let account = google_account_repo.upsert(
        household_id,
        &email,
        display_name.as_deref(),
        avatar_url.as_deref(),
        Some(&access_token),
        refresh_token.as_deref(),
        expires_in,
    ).await?;

    Ok(account)
}
