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

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::MaxTurnsExceeded(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::Provider(_) | AppError::Tool(_) | AppError::Internal(_) => {
                tracing::error!(%self, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
        };

        let body = axum::Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::BadRequest(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
