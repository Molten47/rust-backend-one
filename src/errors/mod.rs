use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Hashing error: {0}")]
    HashingError(String),

    #[error("Token error: {0}")]
    TokenError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::UserAlreadyExists =>
                (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidCredentials =>
                (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::ValidationError(_) =>
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::DatabaseError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::HashingError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::TokenError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Unauthorized(_) =>
               (StatusCode::UNAUTHORIZED, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}