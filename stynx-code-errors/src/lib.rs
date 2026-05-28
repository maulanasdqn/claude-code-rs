use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Max turns ({0}) exceeded")]
    MaxTurnsExceeded(usize),

    #[error("Interrupted")]
    Interrupted,

    #[error("Unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn is_interrupted(&self) -> bool {
        matches!(self, AppError::Interrupted)
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppError::Provider(_) => "provider_error",
            AppError::Tool(_) => "tool_error",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::BadRequest(_) => "bad_request",
            AppError::MaxTurnsExceeded(_) => "max_turns_exceeded",
            AppError::Interrupted => "interrupted",
            AppError::Unauthorized => "unauthorized",
            AppError::Internal(_) => "internal_error",
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            AppError::MaxTurnsExceeded(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Interrupted => StatusCode::OK,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Provider(_) | AppError::Tool(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = self.code();
        let status = self.http_status();
        let public_message = match &self {
            AppError::Provider(_) | AppError::Tool(_) | AppError::Internal(_) => {
                tracing::error!(error.code = %code, error = %self, "internal error");
                "Internal server error".to_string()
            }
            _ => self.to_string(),
        };
        let body = axum::Json(json!({
            "error": public_message,
            "code": code,
        }));
        (status, body).into_response()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::BadRequest(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
