mod errors;
mod middleware;
mod models;
mod routes;
mod cache;
mod handlers;

use axum::{routing::{get, post, patch, delete}, Router, extract::Request, response::Response};
use axum::middleware::{self as axum_middleware, Next};
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

#[derive(Clone)]
pub struct AppState {
    pub pool:               PgPool,
    pub cart_cache:         CartCache,
    pub orders_cache:       OrdersCache,
    pub order_detail_cache: OrderDetailCache,
    pub events_cache:       EventsCache,
}

async fn handle_preflight(req: Request, next: Next) -> Response {
    if req.method() == Method::OPTIONS {
        return Response::builder()
            .status(204)
            .header("access-control-allow-origin", "https://readdeck-app.vercel.app")
            .header("access-control-allow-methods", "GET, POST, PATCH, DELETE, OPTIONS")
            .header("access-control-allow-headers", "content-type, authorization, cookie, accept")
            .header("access-control-allow-credentials", "true")
            .header("access-control-max-age", "3600")
            .body(axum::body::Body::empty())
            .unwrap();
    }
    next.run(req).await
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(1800))
        .idle_timeout(Duration::from_secs(600))
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

    let cors = CorsLayer::new()
        .allow_origin([
            "https://readdeck-app.vercel.app".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([
            Method::GET, Method::POST, Method::PATCH,
            Method::DELETE, Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE, header::AUTHORIZATION,
            header::COOKIE, header::ACCEPT,
        ])
        .expose_headers([header::SET_COOKIE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    let strict_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(20)
            .key_extractor(JwtUserKeyExtractor)
            .finish()
            .expect("Failed to build strict rate limiter"),
    );

    let relaxed_governor = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(200)
            .burst_size(100)
            .key_extractor(JwtUserKeyExtractor)
            .finish()
            .expect("Failed to build relaxed rate limiter"),
    );

    let auth_routes = Router::new()
        .route("/auth/signup",  post(routes::auth::signup))
        .route("/auth/login",   post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/logout",  post(routes::auth::logout))
        .route("/auth/me",      get(routes::auth::me))
        .layer(GovernorLayer { config: strict_governor.clone() });

    let write_routes = Router::new()
        .route("/orders",                post(routes::orders::place_order))
        .route("/orders/:id",            get(routes::orders::get_order).delete(routes::orders::cancel_order))
        .route("/cart",                  get(routes::cart::get_cart).delete(routes::cart::clear_cart))
        .route("/cart/items",            post(routes::cart::add_item))
        .route("/cart/items/:item_id",   patch(routes::cart::update_quantity).delete(routes::cart::remove_item))
        .route("/addresses",             get(routes::address::get_addresses).post(routes::address::create_address))
        .route("/addresses/:id",         patch(routes::address::update_address).delete(routes::address::delete_address))
        .route("/addresses/:id/default", patch(routes::address::set_default_address))
        .route("/vendor/apply",          post(routes::vendor::apply_as_vendor))
        .route("/vendor/bookstore",      post(handlers::vendor_bookstore::create_vendor_bookstore))
        .layer(GovernorLayer { config: strict_governor.clone() });

    let browse_routes = Router::new()
        .route("/orders",                 get(routes::orders::get_orders))
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
        .route("/wishlist",               get(routes::cart::get_wishlist))
        .route("/vendor/status",          get(routes::vendor::get_application_status))
        .layer(GovernorLayer { config: relaxed_governor.clone() });

    let wishlist_write_routes = Router::new()
        .route("/wishlist/items",               post(routes::cart::add_to_wishlist))
        .route("/wishlist/items/:item_id",      delete(routes::cart::remove_from_wishlist))
        .route("/wishlist/items/:item_id/move", post(routes::cart::move_to_cart))
        .layer(GovernorLayer { config: strict_governor.clone() });

    let app = Router::new()
        .merge(auth_routes)
        .merge(write_routes)
        .merge(browse_routes)
        .merge(wishlist_write_routes)
        .with_state(state)
        .layer(axum_middleware::from_fn(handle_preflight))
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server running at http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}