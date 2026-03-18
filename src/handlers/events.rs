use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::events::{EventListResponse, EventQuery, EventResponse},
    AppState,
};

// ── GET /events ───────────────────────────────────────────────────

pub async fn get_events(
    State(state): State<AppState>,
    Query(params): Query<EventQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    let cache_key = format!(
        "events:upcoming={:?}:free={:?}:limit={:?}:offset={:?}",
        params.upcoming, params.free, params.limit, params.offset
    );

    if let Some(cached) = state.events_cache.get(&cache_key).await {
        return Ok(Json(cached));
    }

    let limit  = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT id, title, description, location, bookstore_id,
               starts_at, ends_at, cover_url, is_free, price, created_at
        FROM events
        WHERE
            ($1::bool IS NULL OR (starts_at > now()) = $1)
            AND ($2::bool IS NULL OR is_free = $2)
        ORDER BY starts_at ASC
        LIMIT $3 OFFSET $4
        "#,
        params.upcoming,
        params.free,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM events
           WHERE ($1::bool IS NULL OR (starts_at > now()) = $1)
           AND ($2::bool IS NULL OR is_free = $2)"#,
        params.upcoming,
        params.free
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    let events = rows
        .into_iter()
        .map(|r| EventResponse {
            id:           r.id,
            title:        r.title,
            description:  r.description,
            location:     r.location,
            bookstore_id: r.bookstore_id,
            starts_at:    r.starts_at,
            ends_at:      r.ends_at,
            cover_url:    r.cover_url,
            is_free:      r.is_free,
            price:        r.price.map(|p: sqlx::types::Decimal| p.to_string().parse::<f64>().unwrap_or(0.0)),
        })
        .collect();

    let response = EventListResponse { events, total };

    state.events_cache.insert(cache_key, response.clone()).await;

    Ok(Json(response))
}

// ── GET /events/:id ───────────────────────────────────────────────

pub async fn get_event(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventResponse>, AppError> {
    let r = sqlx::query!(
        r#"
        SELECT id, title, description, location, bookstore_id,
               starts_at, ends_at, cover_url, is_free, price, created_at
        FROM events
        WHERE id = $1
        "#,
        event_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::ValidationError("Event not found".into()))?;

    Ok(Json(EventResponse {
        id:           r.id,
        title:        r.title,
        description:  r.description,
        location:     r.location,
        bookstore_id: r.bookstore_id,
        starts_at:    r.starts_at,
        ends_at:      r.ends_at,
        cover_url:    r.cover_url,
        is_free:      r.is_free,
        price:        r.price.map(|p: sqlx::types::Decimal| p.to_string().parse::<f64>().unwrap_or(0.0)),
    }))
}