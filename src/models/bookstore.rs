use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id: Uuid,
    pub city: String,
    pub district: String,
    pub state: String,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bookstore {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub location_id: Uuid,
    pub rating: Decimal,
    pub total_reviews: i32,
    pub delivery_time_minutes: i32,
    pub delivery_fee: Decimal,
    pub minimum_order: Decimal,
    pub is_open: bool,
    pub image_emoji: String,
    pub created_at: DateTime<Utc>,
}

// Bookstore with location joined
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookstoreWithLocation {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub rating: Decimal,
    pub total_reviews: i32,
    pub delivery_time_minutes: i32,
    pub delivery_fee: Decimal,
    pub minimum_order: Decimal,
    pub is_open: bool,
    pub image_emoji: String,
    pub city: String,
    pub district: String,
    pub state: String,
}