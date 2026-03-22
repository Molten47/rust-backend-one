// src/handlers/vendor_bookstore.rs

use axum::{extract::State, http::StatusCode, Json};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::bookstore::{CreateBookstoreRequest, CreateBookstoreResponse},
    AppState,
};

fn extract_user_id(auth: &AuthUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&auth.0.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))
}

pub async fn create_vendor_bookstore(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateBookstoreRequest>,
) -> Result<(StatusCode, Json<CreateBookstoreResponse>), AppError> {
    let user_id = extract_user_id(&auth)?;

    // Verify caller is a vendor or admin
    let role = sqlx::query_scalar!(
        "SELECT role FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    if role != "vendor" && role != "admin" {
        return Err(AppError::Forbidden("Only vendors can create a bookstore".into()));
    }

    // Block duplicate — one vendor, one store
    let existing = sqlx::query_scalar!(
        "SELECT id FROM bookstores WHERE owner_id = $1 LIMIT 1",
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError(
            "You already have a bookstore registered.".into()
        ));
    }

    // Resolve or create the location row
    let location_id = sqlx::query_scalar!(
        r#"
        INSERT INTO locations (city, district, state, country)
        VALUES (INITCAP($1), INITCAP($2), INITCAP($1), 'Nigeria')
        ON CONFLICT (city, district) DO UPDATE
            SET city = EXCLUDED.city
        RETURNING id
        "#,
        body.city.trim(),
        body.district.trim()
    )
    .fetch_one(&state.pool)
    .await?;

    let delivery_fee  = Decimal::from_str(&body.delivery_fee.to_string())
        .unwrap_or(Decimal::ZERO);
    let minimum_order = Decimal::from_str(&body.minimum_order.to_string())
        .unwrap_or(Decimal::ZERO);

    // Use Decimal::ZERO explicitly for rating — avoids f64/NUMERIC type mismatch
    let zero_decimal  = Decimal::ZERO;
    let zero_int: i32 = 0;

    let row = sqlx::query!(
        r#"
        INSERT INTO bookstores (
            name, address, location_id, owner_id,
            delivery_fee, minimum_order, delivery_time_minutes,
            image_emoji, banner_color,
            description, genres, instagram, website, opening_hours,
            rating, total_reviews,
            is_open, is_published, is_verified
        )
        VALUES (
            $1,  $2,  $3,  $4,
            $5,  $6,  $7,
            $8,  $9,
            $10, $11, $12, $13, $14,
            $15, $16,
            true, true, false
        )
        RETURNING id, name, is_published
        "#,
        body.name.trim(),
        body.address.trim(),
        location_id,
        user_id,
        delivery_fee,
        minimum_order,
        body.delivery_time_minutes,
        body.image_emoji.trim(),
        body.banner_color.trim(),
        body.description.trim(),
        &body.genres,
        body.instagram.as_deref(),
        body.website.as_deref(),
        body.opening_hours.trim(),
        zero_decimal,   // rating — Decimal::ZERO instead of 0.0 literal
        zero_int,       // total_reviews — i32 instead of bare 0
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(
        "Vendor {} created bookstore '{}' ({})",
        user_id, row.name, row.id
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateBookstoreResponse {
            id:           row.id,
            name:         row.name,
            city:         body.city,
            is_published: row.is_published,
        }),
    ))
}