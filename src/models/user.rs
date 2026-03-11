use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// The full database row
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// What the client sends to /auth/signup
#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(
        length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"),
        custom(function = "validate_username")
    )]
    pub username: String,

    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,

    #[validate(
        length(min = 8, message = "Password must be at least 8 characters"),
        custom(function = "validate_password_strength")
    )]
    pub password: String,
}

// What the client sends to /auth/login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Please provide a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

// What we send back on success
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
}

// JWT claims — what gets encoded in the token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

// ── VALIDATORS ───────────────────────────────────────────────────────────────



// Username — only letters, numbers, underscores
fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    if username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(validator::ValidationError::new(
            "Username can only contain letters, numbers and underscores"
        ))
    }
}

// Password — must have uppercase, lowercase, and a number
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_number = password.chars().any(|c| c.is_numeric());

    if has_uppercase && has_lowercase && has_number {
        Ok(())
    } else {
        Err(validator::ValidationError::new(
            "Password must contain at least one uppercase letter, one lowercase letter, and one number"
        ))
    }
}