use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("want {0} not found")]
    WantNotFound(i64),

    #[error("category {0} not found")]
    #[allow(dead_code)]
    CategoryNotFound(i32),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("broker error: {0}")]
    Broker(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::WantNotFound(_) => {
                (StatusCode::NOT_FOUND, "WANT_NOT_FOUND", self.to_string())
            }
            AppError::CategoryNotFound(_) => {
                (StatusCode::NOT_FOUND, "CATEGORY_NOT_FOUND", self.to_string())
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "unauthorized".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "forbidden".to_string(),
            ),
            AppError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_REQUEST", msg.clone())
            }
            AppError::InvalidPayload(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_PAYLOAD", msg.clone())
            }
            AppError::Database(_) | AppError::Broker(_) | AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "internal server error".to_string(),
            ),
        };

        let body = json!({
            "ok": false,
            "error": { "code": code, "message": message }
        });

        (status, Json(body)).into_response()
    }
}
