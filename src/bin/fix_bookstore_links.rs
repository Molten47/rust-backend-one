// src/bin/fix_bookstore_links.rs
// Unlinks ALL vendor-owned bookstores so the setup wizard can run clean

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

    // Show current state
    let linked = sqlx::query!(
        "SELECT bs.name, u.username, u.email
         FROM bookstores bs
         JOIN users u ON u.id = bs.owner_id
         WHERE bs.owner_id IS NOT NULL"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    if linked.is_empty() {
        println!("No linked bookstores found — already clean.");
    } else {
        println!("Currently linked bookstores:");
        for r in &linked {
            println!("  '{}' → {} ({})", r.name, r.username, r.email);
        }

        // Unlink all
        sqlx::query!("UPDATE bookstores SET owner_id = NULL WHERE owner_id IS NOT NULL")
            .execute(&pool)
            .await
            .unwrap();
        println!("\n✅ Unlinked {} bookstore(s)", linked.len());
    }

    println!("\nUsers with vendor role:");
    let vendors = sqlx::query!(
        "SELECT username, email, role FROM users WHERE role != 'reader'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for v in &vendors {
        println!("  {} ({}) — {}", v.username, v.email, v.role);
    }

    println!("\nDone. Vendor users can now run the setup wizard fresh.");
}