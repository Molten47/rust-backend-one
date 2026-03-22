use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await.unwrap();

    // Replace with your actual email
    sqlx::query("UPDATE users SET role = 'vendor' WHERE email = $1")
        .bind("randellbark90@gmail.com")
        .execute(&pool).await.unwrap();

    println!("✅ User promoted to vendor");
}