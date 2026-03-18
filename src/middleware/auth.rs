use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use tower_cookies::Cookies;

use crate::{errors::AppError, models::user::Claims};

pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {

        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized("Could not read cookies".into()))?;

        let token = cookies
            .get("access_token")
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized("No access token cookie".into()))?;

        let secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::TokenError("JWT_SECRET not set".into()))?;

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

        Ok(AuthUser(token_data.claims))
    }
}