use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::vendor::{VendorApplicationRequest, VendorApplicationResponse},
    AppState,
};

fn extract_user_id(auth: &AuthUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&auth.0.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))
}

// ── POST /vendor/apply ────────────────────────────────────────────

pub async fn apply_as_vendor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<VendorApplicationRequest>,
) -> Result<(StatusCode, Json<VendorApplicationResponse>), AppError> {
    let user_id = extract_user_id(&auth)?;

    let existing = sqlx::query_scalar!(
        r#"
        SELECT status FROM vendor_applications
        WHERE user_id = $1
        ORDER BY submitted_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(status) = existing {
        if status == "pending" || status == "reviewing" || status == "approved" {
            return Err(AppError::ValidationError(format!(
                "You already have an application with status: {}",
                status
            )));
        }
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO vendor_applications
            (user_id, store_name, store_address, city, phone, description, instagram, website)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, store_name, store_address, city, status, submitted_at, reviewed_at
        "#,
        user_id,
        body.store_name,
        body.store_address,
        body.city,
        body.phone,
        body.description,
        body.instagram,
        body.website
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(VendorApplicationResponse {
            id:            row.id,
            store_name:    row.store_name,
            store_address: row.store_address,
            city:          row.city,
            status:        row.status,
            submitted_at:  row.submitted_at,
            reviewed_at:   row.reviewed_at,
        }),
    ))
}

// ── GET /vendor/status ────────────────────────────────────────────
// Auto-approves after 60s — restart-safe, no Tokio tasks.
// Only flips role to vendor. Bookstore creation is handled
// separately by the setup wizard (POST /vendor/bookstore).

pub async fn get_application_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<VendorApplicationResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    let row = sqlx::query!(
        r#"
        SELECT id, store_name, store_address, city, status, submitted_at, reviewed_at
        FROM vendor_applications
        WHERE user_id = $1
        ORDER BY submitted_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("No application found".into()))?;

    // Auto-approve after 60 seconds
    if row.status == "pending" {
        let elapsed = Utc::now()
            .signed_duration_since(row.submitted_at)
            .num_seconds();

        if elapsed >= 60 {
            tracing::info!(
                "Auto-approving application {} after {}s",
                row.id, elapsed
            );

            // 1. Mark application approved
            sqlx::query!(
                "UPDATE vendor_applications
                 SET status = 'approved', reviewed_at = NOW()
                 WHERE id = $1",
                row.id
            )
            .execute(&state.pool)
            .await?;

            // 2. Flip user role to vendor
            // No bookstore link here — the setup wizard handles that
            sqlx::query!(
                "UPDATE users SET role = 'vendor' WHERE id = $1",
                user_id
            )
            .execute(&state.pool)
            .await?;

            return Ok(Json(VendorApplicationResponse {
                id:            row.id,
                store_name:    row.store_name,
                store_address: row.store_address,
                city:          row.city,
                status:        "approved".to_string(),
                submitted_at:  row.submitted_at,
                reviewed_at:   Some(Utc::now()),
            }));
        }
    }

    Ok(Json(VendorApplicationResponse {
        id:            row.id,
        store_name:    row.store_name,
        store_address: row.store_address,
        city:          row.city,
        status:        row.status,
        submitted_at:  row.submitted_at,
        reviewed_at:   row.reviewed_at,
    }))
}