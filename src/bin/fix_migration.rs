/// cargo run --bin fix_migration
/// Deletes the orphaned migration 4 record from _sqlx_migrations
/// Run once, then delete this file.

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect");

    let result = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 4")
        .execute(&pool)
        .await
        .expect("Failed to delete");

    println!("✅ Deleted {} orphaned migration record(s)", result.rows_affected());
    println!("Now run: sqlx migrate run");
}