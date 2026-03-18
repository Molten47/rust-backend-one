use axum::{extract::State, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    middleware::auth::AuthUser,
    models::user::{AuthResponse, Claims, LoginRequest, RefreshResponse, SignupRequest, User},
};

// ── SIGNUP ───────────────────────────────────────────────────────

pub async fn signup(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, AppError> {

    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await?;

    if existing.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|e| AppError::HashingError(e.to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await?;

    set_auth_cookies(&state.pool, &cookies, &user).await?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

// ── LOGIN ─────────────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {

    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let valid = verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::HashingError(e.to_string()))?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    set_auth_cookies(&state.pool, &cookies, &user).await?;

    Ok(Json(AuthResponse {
        user_id: user.id,
        username: user.username,
        email: user.email,
    }))
}

// ── REFRESH ───────────────────────────────────────────────────────

pub async fn refresh(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Json<RefreshResponse>, AppError> {

    let refresh_token = cookies
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized("No refresh token".into()))?;

    let stored = sqlx::query_as::<_, crate::models::user::RefreshToken>(
        "SELECT * FROM refresh_tokens WHERE token = $1 AND expires_at > NOW()"
    )
    .bind(&refresh_token)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized("Invalid or expired refresh token".into()))?;

    sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
        .bind(stored.id)
        .execute(&state.pool)
        .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(stored.user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    set_auth_cookies(&state.pool, &cookies, &user).await?;

    Ok(Json(RefreshResponse {
        user_id: user.id,
        username: user.username,
    }))
}

// ── LOGOUT ────────────────────────────────────────────────────────

pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {

    if let Some(cookie) = cookies.get("refresh_token") {
        let _ = sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
            .bind(cookie.value())
            .execute(&state.pool)
            .await;
    }

    cookies.remove(Cookie::new("access_token", ""));
    cookies.remove(Cookie::new("refresh_token", ""));

    StatusCode::OK
}

// ── ME ────────────────────────────────────────────────────────────

pub async fn me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID in token".into()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    Ok(Json(serde_json::json!({
        "user_id": user.id,
        "username": user.username,
        "email": user.email,
        "created_at": user.created_at,
    })))
}

// ── HELPER ────────────────────────────────────────────────────────

async fn set_auth_cookies(
    pool: &sqlx::PgPool,
    cookies: &Cookies,
    user: &User,
) -> Result<(), AppError> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::TokenError("JWT_SECRET not set".into()))?;

    let access_exp = Utc::now()
        .checked_add_signed(Duration::minutes(15))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: access_exp,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::TokenError(e.to_string()))?;

    let refresh_token = Uuid::new_v4().to_string();
    let refresh_exp = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap();

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(user.id)
    .bind(&refresh_token)
    .bind(refresh_exp)
    .execute(pool)
    .await?;

    let mut access_cookie = Cookie::new("access_token", access_token);
    access_cookie.set_http_only(true);
    access_cookie.set_path("/");
    access_cookie.set_max_age(time::Duration::minutes(15));

    let mut refresh_cookie = Cookie::new("refresh_token", refresh_token);
    refresh_cookie.set_http_only(true);
    refresh_cookie.set_path("/auth/refresh");
    refresh_cookie.set_max_age(time::Duration::days(7));

    cookies.add(access_cookie);
    cookies.add(refresh_cookie);

    Ok(())
}