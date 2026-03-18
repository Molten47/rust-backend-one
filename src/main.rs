mod errors;
mod middleware;
mod models;
mod routes;
mod cache;
mod handlers;

use axum::{routing::{get, post, patch, delete}, Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::{env, sync::Arc, time::Duration};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use axum::http::{HeaderValue, Method, header};
use tracing_subscriber;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use cache::{CartCache, OrdersCache, OrderDetailCache, EventsCache};
use middleware::rate_limit::JwtUserKeyExtractor;

// ── App State ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool:               PgPool,
    pub cart_cache:         CartCache,
    pub orders_cache:       OrdersCache,
    pub order_detail_cache: OrderDetailCache,
    pub events_cache:       EventsCache,
}

// ── Main ──────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    // ── DB pool with tuned settings ───────────────────────────────
    // acquire_timeout: fail fast if pool exhausted rather than hang
    // max_lifetime:    recycle connections to avoid stale state
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .max_lifetime(Duration::from_secs(1800))  // 30 min
        .idle_timeout(Duration::from_secs(600))   // 10 min
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Connected to database");

    let state = AppState {
        pool,
        cart_cache:         cache::new_cart_cache(),
        orders_cache:       cache::new_orders_cache(),
        order_detail_cache: cache::new_order_detail_cache(),
        events_cache:       cache::new_events_cache(),
    };

    // ── CORS ──────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::COOKIE])
        .expose_headers([header::SET_COOKIE])
        .allow_credentials(true);

    // ── Rate limiters ─────────────────────────────────────────────
    //
    // STRICT — auth + write routes (orders, cart)
    // 30 requests per second, burst of 20
    // Tight enough to block bots, loose enough for real users
    let strict_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(20)
            .key_extractor(JwtUserKeyExtractor)
            .finish()
            .expect("Failed to build strict rate limiter"),
    );

    // RELAXED — browse routes (books, bookstores, events)
    // 120 requests per second, burst of 40
    // Allows fast browsing/search without blocking legitimate users
    let relaxed_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(120)
            .burst_size(40)
            .key_extractor(JwtUserKeyExtractor)
            .finish()
            .expect("Failed to build relaxed rate limiter"),
    );

    // ── Route groups with per-group rate limiting ─────────────────

    // Auth routes — strict limit (brute force protection)
    let auth_routes = Router::new()
        .route("/auth/signup",  post(routes::auth::signup))
        .route("/auth/login",   post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/logout",  post(routes::auth::logout))
        .route("/auth/me",      get(routes::auth::me))
        .layer(GovernorLayer { config: strict_governor.clone() });

    // Write routes — strict limit (prevent order/cart spam)
    let write_routes = Router::new()
        .route("/orders",              get(routes::orders::get_orders).post(routes::orders::place_order))
        .route("/orders/:id",          get(routes::orders::get_order).delete(routes::orders::cancel_order))
        .route("/cart",                get(routes::cart::get_cart).delete(routes::cart::clear_cart))
        .route("/cart/items",          post(routes::cart::add_item))
        .route("/cart/items/:item_id", patch(routes::cart::update_quantity).delete(routes::cart::remove_item))
        .route("/addresses",             get(routes::address::get_addresses).post(routes::address::create_address))
        .route("/addresses/:id",         patch(routes::address::update_address).delete(routes::address::delete_address))
        .route("/addresses/:id/default", patch(routes::address::set_default_address))
        .route("/vendor/apply",          post(routes::vendor::apply_as_vendor))
        .route("/vendor/status",         get(routes::vendor::get_application_status))
        .layer(GovernorLayer { config: strict_governor.clone() });

    // Browse routes — relaxed limit (search, discovery)
    let browse_routes = Router::new()
        .route("/books",                  get(routes::books::get_books))
        .route("/books/:id",              get(routes::books::get_book))
        .route("/books/:id/availability", get(routes::books::get_book_availability))
        .route("/categories",             get(routes::books::get_categories))
        .route("/bookstores",             get(routes::bookstores::get_bookstores))
        .route("/bookstores/:id",         get(routes::bookstores::get_bookstore))
        .route("/bookstores/:id/books",   get(routes::bookstores::get_bookstore_books))
        .route("/locations",              get(routes::bookstores::get_locations))
        .route("/events",                 get(routes::events::get_events))
        .route("/events/:id",             get(routes::events::get_event))
        .route("/wishlist",                 get(routes::cart::get_wishlist))
        .layer(GovernorLayer { config: relaxed_governor.clone() });

    // Wishlist write — strict
    let wishlist_write_routes = Router::new()
        .route("/wishlist/items",               post(routes::cart::add_to_wishlist))
        .route("/wishlist/items/:item_id",      delete(routes::cart::remove_from_wishlist))
        .route("/wishlist/items/:item_id/move", post(routes::cart::move_to_cart))
        .layer(GovernorLayer { config: strict_governor.clone() });

    // ── Assemble app with global middleware ───────────────────────
    let app = Router::new()
        .merge(auth_routes)
        .merge(write_routes)
        .merge(browse_routes)
        .merge(wishlist_write_routes)
        .with_state(state)
        .layer(CookieManagerLayer::new())
        .layer(cors)
        // Compression: gzip + brotli — cuts JSON 60-80%
        // Especially impactful on Nigerian mobile networks
        .layer(CompressionLayer::new())
        // Request tracing — every request gets a span with method + URI
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Server running at http://localhost:3000");
    println!();
    println!("   ── Auth (strict: 10/s, burst 5) ──────");
    println!("   POST   /auth/signup");
    println!("   POST   /auth/login");
    println!("   POST   /auth/refresh");
    println!("   POST   /auth/logout");
    println!("   GET    /auth/me");
    println!();
    println!("   ── Books (relaxed: 60/s, burst 20) ───");
    println!("   GET    /books  [paginated: ?limit=&cursor=]");
    println!("   GET    /books/:id");
    println!("   GET    /books/:id/availability");
    println!("   GET    /categories");
    println!();
    println!("   ── Bookstores ────────────────────────");
    println!("   GET    /bookstores");
    println!("   GET    /bookstores/:id");
    println!("   GET    /bookstores/:id/books");
    println!("   GET    /locations");
    println!();
    println!("   ── Cart (strict: 10/s, burst 5) ──────");
    println!("   GET    /cart");
    println!("   POST   /cart/items");
    println!("   PATCH  /cart/items/:item_id");
    println!("   DELETE /cart/items/:item_id");
    println!("   DELETE /cart");
    println!();
    println!("   ── Wishlist ──────────────────────────");
    println!("   GET    /wishlist");
    println!("   POST   /wishlist/items");
    println!("   DELETE /wishlist/items/:item_id");
    println!("   POST   /wishlist/items/:item_id/move");
    println!();
    println!("   ── Orders (strict) [paginated] ───────");
    println!("   GET    /orders  [?limit=&cursor=&status=]");
    println!("   POST   /orders");
    println!("   GET    /orders/:id");
    println!("   DELETE /orders/:id  (cancel)");
    println!();
    println!("   ── Events ────────────────────────────");
    println!("   GET    /events");
    println!("   GET    /events/:id");
    println!();
    println!("   ── Vendor ────────────────────────────");
    println!("   POST   /vendor/apply");
    println!("   GET    /vendor/status");
    println!();
    println!("   ── Addresses (strict) ────────────────");
    println!("   GET    /addresses");
    println!("   POST   /addresses");
    println!("   PATCH  /addresses/:id");
    println!("   DELETE /addresses/:id");
    println!("   PATCH  /addresses/:id/default");

    axum::serve(listener, app).await.unwrap();
}