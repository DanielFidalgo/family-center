use poem_openapi::{
    ApiResponse, Object, SecurityScheme, Tags,
    auth::Bearer,
    payload::Json,
};

/// OpenAPI tags for grouping endpoints in the docs UI.
#[derive(Tags)]
pub enum ApiTags {
    /// Authentication & household bootstrap
    Auth,
    /// Google OAuth & calendar management
    Google,
    /// Calendar sync operations
    Sync,
    /// Schedule & events
    Schedule,
    /// People / household members
    People,
    /// Local activities
    Activities,
    /// Household settings
    Settings,
    /// Lane assignment rules
    Lanes,
    /// Person claim (self-service profile setup)
    Claim,
}
use serde::{Deserialize, Serialize};

/// Bearer token security scheme — appears as lock icon in Scalar UI.
#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
pub struct BearerAuth(pub Bearer);

/// Standard error response body.
#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Unified HTTP error responses for all API handlers.
#[derive(ApiResponse)]
pub enum ApiError {
    /// 400 Bad Request
    #[oai(status = 400)]
    BadRequest(Json<ErrorBody>),
    /// 401 Unauthorized
    #[oai(status = 401)]
    Unauthorized(Json<ErrorBody>),
    /// 404 Not Found
    #[oai(status = 404)]
    NotFound(Json<ErrorBody>),
    /// 500 Internal Server Error
    #[oai(status = 500)]
    Internal(Json<ErrorBody>),
    /// 410 Gone
    #[oai(status = 410)]
    Gone(Json<ErrorBody>),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(Json(ErrorBody {
            code: "BAD_REQUEST".to_string(),
            message: msg.into(),
        }))
    }

    pub fn unauthorized() -> Self {
        ApiError::Unauthorized(Json(ErrorBody {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
        }))
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(Json(ErrorBody {
            code: "NOT_FOUND".to_string(),
            message: msg.into(),
        }))
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(Json(ErrorBody {
            code: "INTERNAL_ERROR".to_string(),
            message: msg.into(),
        }))
    }

    pub fn gone(msg: impl Into<String>) -> Self {
        ApiError::Gone(Json(ErrorBody {
            code: "GONE".to_string(),
            message: msg.into(),
        }))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("Database error: {e}");
        ApiError::internal("Database error")
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("Internal error: {e}");
        ApiError::internal("Internal server error")
    }
}

impl From<crate::domain::error::DomainError> for ApiError {
    fn from(e: crate::domain::error::DomainError) -> Self {
        match e {
            crate::domain::error::DomainError::NotFound(msg) => ApiError::not_found(msg),
            crate::domain::error::DomainError::Validation(msg) => ApiError::bad_request(msg),
            crate::domain::error::DomainError::Unauthorized => ApiError::unauthorized(),
        }
    }
}
