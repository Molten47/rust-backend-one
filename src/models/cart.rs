use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;



// ── Request types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub book_id:  Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuantityRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct AddToWishlistRequest {
    pub book_id: Uuid,
}

// ── Response types (also cached) ─────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItemResponse {
    pub id:             Uuid,
    pub book_id:        Uuid,
    pub title:          String,
    pub author:         String,
    pub price_snapshot: f64,
    pub quantity:       i32,
    pub subtotal:       f64,
    // NEW fields:
    pub bookstore_id:   Uuid,
    pub bookstore_name: String,
    pub cover_emoji:    Option<String>,
    pub cover_color:    Option<String>,
    pub cover_url:      Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartResponse {
    pub id:          Uuid,
    pub status:      String,
    pub items:       Vec<CartItemResponse>,
    pub total:       f64,
    pub item_count:  i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WishlistItemResponse {
    pub id:      Uuid,
    pub book_id: Uuid,
    pub title:   String,
    pub author:  String,
    pub price:   f64,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WishlistResponse {
    pub id:    Uuid,
    pub items: Vec<WishlistItemResponse>,
}