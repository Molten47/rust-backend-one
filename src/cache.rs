use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use crate::models::cart::CartResponse;
use crate::models::orders::{OrderListResponse, OrderResponse};
use crate::models::events::EventListResponse;

// ── Cart Cache ────────────────────────────────────────────────────

pub type CartCache = Arc<Cache<Uuid, CartResponse>>;

pub fn new_cart_cache() -> CartCache {
    Arc::new(
        Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(120))
            .build(),
    )
}

// ── Orders Cache ──────────────────────────────────────────────────
// Keyed by user_id — caches the full order list per user.
// Short TTL since order status changes frequently.

pub type OrdersCache = Arc<Cache<Uuid, OrderListResponse>>;

pub fn new_orders_cache() -> OrdersCache {
    Arc::new(
        Cache::builder()
            .max_capacity(5_000)
            .time_to_live(Duration::from_secs(60))   // 1 min TTL — status updates often
            .time_to_idle(Duration::from_secs(30))
            .build(),
    )
}

// ── Order Detail Cache ────────────────────────────────────────────
// Keyed by order_id — caches individual order detail.

pub type OrderDetailCache = Arc<Cache<Uuid, OrderResponse>>;

pub fn new_order_detail_cache() -> OrderDetailCache {
    Arc::new(
        Cache::builder()
            .max_capacity(20_000)
            .time_to_live(Duration::from_secs(30))   // 30s — tracking needs freshness
            .time_to_idle(Duration::from_secs(15))
            .build(),
    )
}

// ── Events Cache ──────────────────────────────────────────────────
// Keyed by a simple string key e.g. "upcoming", "free", "all".
// Events list rarely changes — longer TTL is fine.

pub type EventsCache = Arc<Cache<String, EventListResponse>>;

pub fn new_events_cache() -> EventsCache {
    Arc::new(
        Cache::builder()
            .max_capacity(100)                        // small — few distinct queries
            .time_to_live(Duration::from_secs(300))   // 5 min TTL
            .time_to_idle(Duration::from_secs(180))
            .build(),
    )
}