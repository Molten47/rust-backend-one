use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    models::book::{BookWithDetails, Category},
};

#[derive(Debug, Deserialize)]
pub struct BookQuery {
    pub category:  Option<String>,
    pub city:      Option<String>,
    pub bookstore: Option<String>,
    /// Cursor — rating of the last book on the previous page (DESC order)
    pub cursor:    Option<f64>,
    /// Max results (default 20, max 50)
    pub limit:     Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BookListResponse {
    pub books:       Vec<BookWithDetails>,
    pub has_more:    bool,
    pub next_cursor: Option<f64>,
    pub limit:       i64,
}

// GET /books
pub async fn get_books(
    State(state): State<AppState>,
    Query(params): Query<BookQuery>,
) -> Result<Json<BookListResponse>, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 50);

    // Fetch limit+1 to detect next page
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
         WHERE ($1::text IS NULL OR LOWER(c.slug) = LOWER($1))
           AND ($2::text IS NULL OR LOWER(l.city) = LOWER($2))
           AND ($3::text IS NULL OR LOWER(bs.name) = LOWER($3))
           AND bk.in_stock = true
           AND ($4::float8 IS NULL OR bk.rating <= $4)
         ORDER BY bk.rating DESC
         LIMIT $5"
    )
    .bind(&params.category)
    .bind(&params.city)
    .bind(&params.bookstore)
    .bind(params.cursor)
    .bind(limit + 1)
    .fetch_all(&state.pool)
    .await?;

    let has_more = books.len() as i64 > limit;
    let page = if has_more { &books[..limit as usize] } else { &books[..] };

    // next_cursor is the rating of the last book on this page
    let next_cursor = if has_more {
        page.last().map(|b| {
            b.rating.to_string().parse::<f64>().unwrap_or(0.0)
        })
    } else {
        None
    };

    Ok(Json(BookListResponse {
        books: page.to_vec(),
        has_more,
        next_cursor,
        limit,
    }))
}

// GET /books/:id
pub async fn get_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {

    let book = sqlx::query_as::<_, BookWithDetails>(
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
         WHERE bk.id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Book not found".into()))?;

    Ok(Json(serde_json::json!({ "book": book })))
}

// GET /categories
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories ORDER BY name"
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "categories": categories })))
}

// ── Availability ──────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BookstoreAvailability {
    pub bookstore_id:       Uuid,
    pub bookstore_name:     String,
    pub address:            String,
    pub city:               String,
    pub is_open:            bool,
    pub delivery_time_mins: i32,
    pub delivery_fee:       rust_decimal::Decimal,
    pub price:              rust_decimal::Decimal,
    pub in_stock:           bool,
    pub rating:             rust_decimal::Decimal,
}

// GET /books/:id/availability
pub async fn get_book_availability(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {

    let rows = sqlx::query_as::<_, BookstoreAvailability>(
        "SELECT
            bs.id        AS bookstore_id,
            bs.name      AS bookstore_name,
            bs.address,
            l.city,
            bs.is_open,
            bs.delivery_time_minutes AS delivery_time_mins,
            bs.delivery_fee,
            bk.price,
            bk.in_stock,
            bs.rating
         FROM books bk
         JOIN bookstores bs ON bk.bookstore_id = bs.id
         JOIN locations  l  ON bs.location_id  = l.id
         WHERE bk.id = $1
         ORDER BY bs.is_open DESC, bs.rating DESC"
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "availability": rows })))
}