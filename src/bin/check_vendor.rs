// src/bin/check_vendor.rs
// One-shot: prints all vendor applications and the user's current role

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

    println!("── Vendor applications ──────────────────");
    let apps = sqlx::query!(
        "SELECT id, user_id, store_name, status, submitted_at, reviewed_at
         FROM vendor_applications
         ORDER BY submitted_at DESC"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for a in &apps {
        println!(
            "  id={} store='{}' status={} submitted={}",
            &a.id.to_string()[..8], a.store_name, a.status,
            a.submitted_at.format("%H:%M:%S")
        );
    }

    println!();
    println!("── Users with vendor/admin role ─────────");
    let users = sqlx::query!(
        "SELECT id, username, email, role FROM users WHERE role != 'reader'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for u in &users {
        println!("  {} ({}) — role={}", u.username, u.email, u.role);
    }

    if apps.is_empty() { println!("  No applications found"); }
    if users.is_empty() { println!("  No vendor/admin users found"); }
}