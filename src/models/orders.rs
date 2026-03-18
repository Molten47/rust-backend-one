use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── DB Row ────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OrderRow {
    pub id:           Uuid,
    pub user_id:      Uuid,
    pub bookstore_id: Uuid,
    pub status:       String,
    pub total_amount: sqlx::types::Decimal,
    pub delivery_fee: sqlx::types::Decimal,
    pub address:      String,
    pub notes:        Option<String>,
    pub placed_at:    DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OrderItemRow {
    pub id:         Uuid,
    pub order_id:   Uuid,
    pub book_id:    Uuid,
    pub quantity:   i32,
    pub unit_price: sqlx::types::Decimal,
    pub title:      Option<String>,
    pub cover_url:  Option<String>,
    pub author:     Option<String>,
}

// ── API Responses ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrderItemResponse {
    pub id:          Uuid,
    pub book_id:     Uuid,
    pub title:       String,
    pub author:      String,
    pub cover_emoji: Option<String>,
    pub cover_color: Option<String>,
    pub quantity:    i32,
    pub unit_price:  f64,
    pub subtotal:    f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id:           Uuid,
    pub bookstore_id: Uuid,
    pub status:       String,
    pub total_amount: f64,
    pub delivery_fee: f64,
    pub address:      String,
    pub notes:        Option<String>,
    pub placed_at:    DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
    pub items:        Vec<OrderItemResponse>,
}

// ── Paginated list response ───────────────────────────────────────

/// Cursor-based pagination — client sends back `next_cursor` as
/// the `cursor` query param on the next request.
/// `next_cursor` is None when there are no more pages.
#[derive(Debug, Clone, Serialize)]
pub struct OrderListResponse {
    pub orders:      Vec<OrderSummary>,
    pub total:       i64,
    /// ISO-8601 timestamp of the last item — pass as ?cursor= for next page
    pub next_cursor: Option<DateTime<Utc>>,
    pub has_more:    bool,
    pub limit:       i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderSummary {
    pub id:           Uuid,
    pub bookstore_id: Uuid,
    pub status:       String,
    pub total_amount: f64,
    pub item_count:   i64,
    pub placed_at:    DateTime<Utc>,
}

// ── Query params ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    /// Max items to return (default 20, max 50)
    pub limit:  Option<i64>,
    /// placed_at timestamp of the last item from previous page
    pub cursor: Option<DateTime<Utc>>,
    /// Optional status filter
    pub status: Option<String>,
}

// ── Request Bodies ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub bookstore_id:  Uuid,
    pub address:       String,
    pub notes:         Option<String>,
    /// Specific cart item IDs — if empty, orders all items from this bookstore
    pub cart_item_ids: Vec<Uuid>,
}