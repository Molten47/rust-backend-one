use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use std::env;

use crate::{
    errors::AppError,
    models::user::{AuthResponse, Claims, LoginRequest, SignupRequest, User},
};

// ── SIGNUP ──────────────────────────────────────────────────────────────────

pub async fn signup(
    State(pool): State<PgPool>,
    Json(body): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, AppError> {

    // 1. Check if email already exists
    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(&pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // 2. Hash the password
    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|e| AppError::HashingError(e.to_string()))?;

    // 3. Insert user into database
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash)
         VALUES ($1, $2, $3)
         RETURNING *"
    )
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    // 4. Generate JWT token
    let token = generate_token(&user)?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

// ── LOGIN ────────────────────────────────────────────────────────────────────

pub async fn login(
    State(pool): State<PgPool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {

    // 1. Find user by email
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::InvalidCredentials)?; // None → error

    // 2. Verify password against stored hash
    let valid = verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::HashingError(e.to_string()))?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    // 3. Generate JWT token
    let token = generate_token(&user)?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

// ── HELPER ───────────────────────────────────────────────────────────────────

fn generate_token(user: &User) -> Result<String, AppError> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::TokenError("JWT_SECRET not set".into()))?;

    let expiry = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: expiry,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::TokenError(e.to_string()))
}