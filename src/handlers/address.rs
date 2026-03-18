use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::address::{
        AddressListResponse, AddressResponse,
        CreateAddressRequest, UpdateAddressRequest,
    },
    AppState,
};

fn extract_user_id(auth: &AuthUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&auth.0.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))
}

// ── GET /addresses ────────────────────────────────────────────────

pub async fn get_addresses(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<AddressListResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    let rows = sqlx::query_as::<_, crate::models::address::SavedAddress>(
        "SELECT * FROM saved_addresses WHERE user_id = $1 ORDER BY is_default DESC, created_at ASC"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    let addresses = rows.into_iter().map(AddressResponse::from).collect();

    Ok(Json(AddressListResponse { addresses }))
}

// ── POST /addresses ───────────────────────────────────────────────

pub async fn create_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateAddressRequest>,
) -> Result<Json<AddressResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let make_default = body.is_default.unwrap_or(false);

    // Count existing addresses — if this is the first one, auto-default it
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM saved_addresses WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    let is_default = make_default || count == 0;

    // If making this default, clear existing default first
    if is_default {
        sqlx::query(
            "UPDATE saved_addresses SET is_default = false WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, crate::models::address::SavedAddress>(
        r#"
        INSERT INTO saved_addresses (user_id, label, address, city, phone, is_default)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&body.label)
    .bind(&body.address)
    .bind(&body.city)
    .bind(&body.phone)
    .bind(is_default)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(AddressResponse::from(row)))
}

// ── PATCH /addresses/:id ──────────────────────────────────────────

pub async fn update_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(address_id): Path<Uuid>,
    Json(body): Json<UpdateAddressRequest>,
) -> Result<Json<AddressResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Verify ownership
    let _existing = sqlx::query_as::<_, crate::models::address::SavedAddress>(
        "SELECT * FROM saved_addresses WHERE id = $1 AND user_id = $2"
    )
    .bind(address_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::ValidationError("Address not found".into()))?;

    // If setting as default, clear others first
    if body.is_default == Some(true) {
        sqlx::query(
            "UPDATE saved_addresses SET is_default = false WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&state.pool)
        .await?;
    }

    let row = sqlx::query_as::<_, crate::models::address::SavedAddress>(
        r#"
        UPDATE saved_addresses SET
            label      = COALESCE($1, label),
            address    = COALESCE($2, address),
            city       = COALESCE($3, city),
            phone      = COALESCE($4, phone),
            is_default = COALESCE($5, is_default),
            updated_at = now()
        WHERE id = $6 AND user_id = $7
        RETURNING *
        "#
    )
    .bind(body.label.as_deref())
    .bind(body.address.as_deref())
    .bind(body.city.as_deref())
    .bind(body.phone.as_deref())
    .bind(body.is_default)
    .bind(address_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(AddressResponse::from(row)))
}

// ── DELETE /addresses/:id ─────────────────────────────────────────

pub async fn delete_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(address_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = extract_user_id(&auth)?;

    let result = sqlx::query(
        "DELETE FROM saved_addresses WHERE id = $1 AND user_id = $2"
    )
    .bind(address_id)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::ValidationError("Address not found".into()));
    }

    // If deleted address was default, promote the most recent remaining one
    sqlx::query(
        r#"
        UPDATE saved_addresses SET is_default = true
        WHERE user_id = $1
        AND id = (
            SELECT id FROM saved_addresses
            WHERE user_id = $1
            ORDER BY created_at ASC
            LIMIT 1
        )
        "#
    )
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ── PATCH /addresses/:id/default ─────────────────────────────────

pub async fn set_default_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(address_id): Path<Uuid>,
) -> Result<Json<AddressResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    // Verify ownership
    sqlx::query(
        "SELECT id FROM saved_addresses WHERE id = $1 AND user_id = $2"
    )
    .bind(address_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::ValidationError("Address not found".into()))?;

    // Clear all defaults for this user
    sqlx::query(
        "UPDATE saved_addresses SET is_default = false WHERE user_id = $1"
    )
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    // Set new default
    let row = sqlx::query_as::<_, crate::models::address::SavedAddress>(
        "UPDATE saved_addresses SET is_default = true, updated_at = now() WHERE id = $1 RETURNING *"
    )
    .bind(address_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(AddressResponse::from(row)))
}