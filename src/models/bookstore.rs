use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id:         Uuid,
    pub city:       String,
    pub district:   String,
    pub state:      String,
    pub country:    String,
    pub latitude:   Option<f64>,
    pub longitude:  Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bookstore {
    pub id:                     Uuid,
    pub name:                   String,
    pub address:                String,
    pub location_id:            Uuid,
    pub owner_id:               Option<Uuid>,
    pub rating:                 Decimal,
    pub total_reviews:          i32,
    pub delivery_time_minutes:  i32,
    pub delivery_fee:           Decimal,
    pub minimum_order:          Decimal,
    pub is_open:                bool,
    pub is_published:           bool,
    pub is_verified:            bool,
    pub image_emoji:            String,
    pub banner_color:           String,
    pub description:            Option<String>,
    pub genres:                 Option<Vec<String>>,
    pub instagram:              Option<String>,
    pub website:                Option<String>,
    pub opening_hours:          Option<String>,
    pub created_at:             DateTime<Utc>,
}

// Bookstore with location joined — used in list/detail endpoints
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookstoreWithLocation {
    pub id:                    Uuid,
    pub name:                  String,
    pub address:               String,
    pub rating:                Decimal,
    pub total_reviews:         i32,
    pub delivery_time_minutes: i32,
    pub delivery_fee:          Decimal,
    pub minimum_order:         Decimal,
    pub is_open:               bool,
    pub image_emoji:           String,
    pub city:                  String,
    pub district:              String,
    pub state:                 String,
}

// ── Vendor bookstore creation ─────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateBookstoreRequest {
    // Step 1 — basics
    pub name:                  String,
    pub address:               String,
    pub city:                  String,
    pub district:              String,

    // Step 2 — identity
    pub description:           String,
    pub genres:                Vec<String>,
    pub image_emoji:           String,
    pub banner_color:          String,

    // Step 3 — operations
    pub delivery_fee:          f64,
    pub minimum_order:         f64,
    pub delivery_time_minutes: i32,
    pub opening_hours:         String,

    // Step 4 — socials (optional)
    pub instagram:             Option<String>,
    pub website:               Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateBookstoreResponse {
    pub id:           Uuid,
    pub name:         String,
    pub city:         String,
    pub is_published: bool,
}