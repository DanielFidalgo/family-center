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
    fn s3_endpoint(&self) -> &str;
    fn s3_bucket(&self) -> &str;
    fn s3_access_key(&self) -> &str;
    fn s3_secret_key(&self) -> &str;
    fn s3_region(&self) -> &str;
    fn public_url(&self) -> &str;
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
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub public_url: String,
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
            s3_endpoint: std::env::var("S3_ENDPOINT").unwrap_or_default(),
            s3_bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "family-center".to_string()),
            s3_access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_default(),
            s3_secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_default(),
            s3_region: std::env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string()),
            public_url: std::env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
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
    fn s3_endpoint(&self) -> &str { &self.s3_endpoint }
    fn s3_bucket(&self) -> &str { &self.s3_bucket }
    fn s3_access_key(&self) -> &str { &self.s3_access_key }
    fn s3_secret_key(&self) -> &str { &self.s3_secret_key }
    fn s3_region(&self) -> &str { &self.s3_region }
    fn public_url(&self) -> &str { &self.public_url }
}
