mod errors;
mod models;
mod routes;

use axum::{routing::post, Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Connect to database
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Connected to database");

    // CORS — allow any origin (fine for dev/portfolio)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/auth/signup", post(routes::auth::signup))
        .route("/auth/login",  post(routes::auth::login))
        .with_state(pool)
        .layer(cors);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running at http://localhost:3000");
    println!("   POST /auth/signup");
    println!("   POST /auth/login");

    axum::serve(listener, app).await.unwrap();
}
