use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

use crate::{errors::AppError, models::user::Claims};

pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers)?;

        let secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::TokenError("JWT_SECRET not set".into()))?;

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::TokenError(e.to_string()))?;

        Ok(AuthUser(token_data.claims))
    }
}

fn extract_token(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AppError::TokenError("Missing Authorization header".into()))?
        .to_str()
        .map_err(|_| AppError::TokenError("Invalid Authorization header".into()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::TokenError("Authorization header must start with 'Bearer '".into()));
    }

    Ok(auth_header[7..].to_string())
}