use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── DB row ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct SavedAddress {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub label:      String,
    pub address:    String,
    pub city:       String,
    pub phone:      String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Request types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAddressRequest {
    #[validate(length(min = 1, max = 50, message = "Label must be between 1 and 50 characters"))]
    pub label: String,

    #[validate(length(min = 5, message = "Address must be at least 5 characters"))]
    pub address: String,

    #[validate(length(min = 1, message = "City is required"))]
    pub city: String,

    #[validate(length(min = 7, message = "Phone number must be at least 7 characters"))]
    pub phone: String,

    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAddressRequest {
    #[validate(length(min = 1, max = 50))]
    pub label: Option<String>,

    #[validate(length(min = 5))]
    pub address: Option<String>,

    #[validate(length(min = 1))]
    pub city: Option<String>,

    #[validate(length(min = 7))]
    pub phone: Option<String>,

    pub is_default: Option<bool>,
}

// ── Response types ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressResponse {
    pub id:         Uuid,
    pub label:      String,
    pub address:    String,
    pub city:       String,
    pub phone:      String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl From<SavedAddress> for AddressResponse {
    fn from(a: SavedAddress) -> Self {
        Self {
            id:         a.id,
            label:      a.label,
            address:    a.address,
            city:       a.city,
            phone:      a.phone,
            is_default: a.is_default,
            created_at: a.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressListResponse {
    pub addresses: Vec<AddressResponse>,
}