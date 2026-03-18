use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id:          Uuid,
    pub name:        String,
    pub slug:        String,
    pub emoji:       String,
    pub description: Option<String>,
    pub created_at:  DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Book {
    pub id:            Uuid,
    pub title:         String,
    pub author:        String,
    pub price:         Decimal,
    pub category_id:   Uuid,
    pub bookstore_id:  Uuid,
    pub description:   Option<String>,
    pub cover_emoji:   String,
    pub cover_color:   String,
    pub in_stock:      bool,
    pub rating:        Decimal,
    pub total_reviews: i32,
    pub created_at:    DateTime<Utc>,
}

// Book with category and bookstore name joined
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookWithDetails {
    pub id:             Uuid,
    pub title:          String,
    pub author:         String,
    pub price:          Decimal,
    pub description:    Option<String>,
    pub cover_url:      Option<String>,  // Real image from Open Library — null falls back to emoji
    pub cover_emoji:    String,
    pub cover_color:    String,
    pub in_stock:       bool,
    pub rating:         Decimal,
    pub total_reviews:  i32,
    pub category_name:  String,
    pub category_slug:  String,
    pub category_emoji: String,
    pub bookstore_name: String,
    pub bookstore_id:   Uuid,
    pub city:           String,
}