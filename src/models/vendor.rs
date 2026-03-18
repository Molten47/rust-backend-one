use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── DB Row ────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct VendorApplicationRow {
    pub id:             Uuid,
    pub user_id:        Uuid,
    pub store_name:     String,
    pub store_address:  String,
    pub city:           String,
    pub phone:          String,
    pub description:    Option<String>,
    pub instagram:      Option<String>,
    pub website:        Option<String>,
    pub status:         String,
    pub submitted_at:   DateTime<Utc>,
    pub reviewed_at:    Option<DateTime<Utc>>,
    pub reviewer_notes: Option<String>,
}

// ── API Response ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct VendorApplicationResponse {
    pub id:           Uuid,
    pub store_name:   String,
    pub store_address: String,
    pub city:         String,
    pub status:       String,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at:  Option<DateTime<Utc>>,
}

// ── Request Body ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VendorApplicationRequest {
    pub store_name:    String,
    pub store_address: String,
    pub city:          String,
    pub phone:         String,
    pub description:   Option<String>,
    pub instagram:     Option<String>,
    pub website:       Option<String>,
}