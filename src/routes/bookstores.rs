use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    models::bookstore::{BookstoreWithLocation, Location},
    models::book::BookWithDetails,
};

#[derive(Debug, Deserialize)]
pub struct BookstoreQuery {
    pub city:     Option<String>,
    pub district: Option<String>,
}

// GET /bookstores
pub async fn get_bookstores(
    State(state): State<AppState>,
    Query(params): Query<BookstoreQuery>,
) -> Result<Json<serde_json::Value>, AppError> {

    let bookstores = match (&params.city, &params.district) {
        (Some(city), Some(district)) => {
            sqlx::query_as::<_, BookstoreWithLocation>(
                "SELECT b.id, b.name, b.address, b.rating, b.total_reviews,
                        b.delivery_time_minutes, b.delivery_fee, b.minimum_order,
                        b.is_open, b.image_emoji,
                        l.city, l.district, l.state
                 FROM bookstores b
                 JOIN locations l ON b.location_id = l.id
                 WHERE LOWER(l.city) = LOWER($1)
                 AND LOWER(l.district) = LOWER($2)
                 ORDER BY b.rating DESC"
            )
            .bind(city)
            .bind(district)
            .fetch_all(&state.pool)
            .await?
        }
        (Some(city), None) => {
            sqlx::query_as::<_, BookstoreWithLocation>(
                "SELECT b.id, b.name, b.address, b.rating, b.total_reviews,
                        b.delivery_time_minutes, b.delivery_fee, b.minimum_order,
                        b.is_open, b.image_emoji,
                        l.city, l.district, l.state
                 FROM bookstores b
                 JOIN locations l ON b.location_id = l.id
                 WHERE LOWER(l.city) = LOWER($1)
                 ORDER BY b.rating DESC"
            )
            .bind(city)
            .fetch_all(&state.pool)
            .await?
        }
        _ => {
            sqlx::query_as::<_, BookstoreWithLocation>(
                "SELECT b.id, b.name, b.address, b.rating, b.total_reviews,
                        b.delivery_time_minutes, b.delivery_fee, b.minimum_order,
                        b.is_open, b.image_emoji,
                        l.city, l.district, l.state
                 FROM bookstores b
                 JOIN locations l ON b.location_id = l.id
                 ORDER BY b.rating DESC"
            )
            .fetch_all(&state.pool)
            .await?
        }
    };

    Ok(Json(serde_json::json!({ "bookstores": bookstores })))
}

// GET /bookstores/:id
pub async fn get_bookstore(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {

    let bookstore = sqlx::query_as::<_, BookstoreWithLocation>(
        "SELECT b.id, b.name, b.address, b.rating, b.total_reviews,
                b.delivery_time_minutes, b.delivery_fee, b.minimum_order,
                b.is_open, b.image_emoji,
                l.city, l.district, l.state
         FROM bookstores b
         JOIN locations l ON b.location_id = l.id
         WHERE b.id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Bookstore not found".into()))?;

    Ok(Json(serde_json::json!({ "bookstore": bookstore })))
}

// GET /bookstores/:id/books
pub async fn get_bookstore_books(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {

    // Verify bookstore exists first
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM bookstores WHERE id = $1)",
        id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::ValidationError("Bookstore not found".into()));
    }

    let books = sqlx::query_as::<_, BookWithDetails>(
        "SELECT bk.id, bk.title, bk.author, bk.price, bk.description,
                bk.cover_url, bk.cover_emoji, bk.cover_color, bk.in_stock, bk.rating,
                bk.total_reviews, bk.bookstore_id,
                c.name as category_name, c.slug as category_slug,
                c.emoji as category_emoji,
                bs.name as bookstore_name,
                l.city
         FROM books bk
         JOIN categories c ON bk.category_id = c.id
         JOIN bookstores bs ON bk.bookstore_id = bs.id
         JOIN locations l ON bs.location_id = l.id
         WHERE bk.bookstore_id = $1
         AND bk.in_stock = true
         ORDER BY bk.rating DESC"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "books": books })))
}

// GET /locations
pub async fn get_locations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {

    let locations = sqlx::query_as::<_, Location>(
        "SELECT * FROM locations ORDER BY city, district"
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "locations": locations })))
}