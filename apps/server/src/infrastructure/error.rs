use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("external api error: {0}")]
    ExternalApi(String),
}
