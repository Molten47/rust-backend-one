use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── DB Row ────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct EventRow {
    pub id:           Uuid,
    pub title:        String,
    pub description:  Option<String>,
    pub location:     String,
    pub bookstore_id: Option<Uuid>,
    pub starts_at:    DateTime<Utc>,
    pub ends_at:      Option<DateTime<Utc>>,
    pub cover_url:    Option<String>,
    pub is_free:      bool,
    pub price:        Option<sqlx::types::Decimal>,
    pub created_at:   DateTime<Utc>,
}

// ── API Response ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct EventResponse {
    pub id:           Uuid,
    pub title:        String,
    pub description:  Option<String>,
    pub location:     String,
    pub bookstore_id: Option<Uuid>,
    pub starts_at:    DateTime<Utc>,
    pub ends_at:      Option<DateTime<Utc>>,
    pub cover_url:    Option<String>,
    pub is_free:      bool,
    pub price:        Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total:  i64,
}

// ── Query Params ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub upcoming: Option<bool>,
    pub free:     Option<bool>,
    pub limit:    Option<i64>,
    pub offset:   Option<i64>,
}