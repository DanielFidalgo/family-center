use anyhow::{Context, Result};

pub trait IAppConfig: Send + Sync {
    fn database_url(&self) -> &str;
    fn server_host(&self) -> &str;
    fn server_port(&self) -> u16;
    fn jwt_secret(&self) -> &str;
    fn google_client_id(&self) -> &str;
    fn google_client_secret(&self) -> &str;
    fn google_redirect_uri(&self) -> &str;
    fn mock_calendar(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub mock_calendar: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("SERVER_PORT must be a valid port number")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/google/callback".to_string()),
            mock_calendar: std::env::var("MOCK_CALENDAR")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        })
    }
}

impl IAppConfig for Config {
    fn database_url(&self) -> &str {
        &self.database_url
    }
    fn server_host(&self) -> &str {
        &self.server_host
    }
    fn server_port(&self) -> u16 {
        self.server_port
    }
    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
    fn google_client_id(&self) -> &str {
        &self.google_client_id
    }
    fn google_client_secret(&self) -> &str {
        &self.google_client_secret
    }
    fn google_redirect_uri(&self) -> &str {
        &self.google_redirect_uri
    }
    fn mock_calendar(&self) -> bool {
        self.mock_calendar
    }
}
