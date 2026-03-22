// src/bin/reset_vendor.rs
// Cleans up stuck vendor applications and resets for a fresh test

use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    // Delete all pending applications so the user can apply fresh
    let deleted = sqlx::query!(
        "DELETE FROM vendor_applications WHERE status = 'pending' RETURNING id"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    println!("🗑  Deleted {} pending application(s)", deleted.len());

    // Reset user role back to reader so the full journey can be tested
    sqlx::query!(
        "UPDATE users SET role = 'reader' WHERE email = 'randellbark90@gmail.com'"
    )
    .execute(&pool)
    .await
    .unwrap();
    println!("👤 Reset Muhamme47 role → reader");

    // Unlink bookstore so setup wizard runs again
    sqlx::query!(
        "UPDATE bookstores SET owner_id = NULL WHERE owner_id = (
            SELECT id FROM users WHERE email = 'randellbark90@gmail.com'
        )"
    )
    .execute(&pool)
    .await
    .unwrap();
    println!("🏪 Unlinked bookstore from user");

    println!();
    println!("✅ Ready for a clean test. Log out and log back in on the frontend.");
}