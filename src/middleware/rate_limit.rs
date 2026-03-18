use std::net::IpAddr;
use axum::extract::Request;
use tower_governor::key_extractor::KeyExtractor;
use tower_governor::GovernorError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::env;

/// Extracts the user ID from the JWT access_token cookie.
/// Falls back to IP address for unauthenticated requests.
/// Per-user rate limiting survives NAT and shared carrier IPs —
/// critical for Nigerian mobile networks where many users share one IP.
#[derive(Clone, Debug)]
pub struct JwtUserKeyExtractor;

impl KeyExtractor for JwtUserKeyExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        // Try to get user ID from JWT cookie first
        if let Some(cookie_header) = req.headers().get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for part in cookie_str.split(';') {
                    let part = part.trim();
                    if let Some(token) = part.strip_prefix("access_token=") {
                        if let Ok(secret) = env::var("JWT_SECRET") {
                            let mut validation = Validation::new(Algorithm::HS256);
                            validation.validate_exp = false;
                            if let Ok(data) = decode::<serde_json::Value>(
                                token,
                                &DecodingKey::from_secret(secret.as_bytes()),
                                &validation,
                            ) {
                                if let Some(sub) = data.claims.get("sub").and_then(|v| v.as_str()) {
                                    return Ok(format!("user:{}", sub));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fall back to IP address for unauthenticated requests
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::from([127, 0, 0, 1]));

        Ok(format!("ip:{}", ip))
    }
}