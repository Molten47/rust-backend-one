use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct BookstoreRow {
    id:   Uuid,
    name: String,
    city: String,
}

#[derive(Debug, sqlx::FromRow)]
struct VendorRow {
    id:       Uuid,
    username: String,
    email:    String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let bookstores = sqlx::query_as::<_, BookstoreRow>(
        "SELECT bs.id, bs.name, l.city
         FROM bookstores bs
         JOIN locations l ON bs.location_id = l.id
         ORDER BY bs.name"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    println!("Available bookstores:");
    for (i, b) in bookstores.iter().enumerate() {
        println!("  [{}] {}  {}", i, b.name, b.city);
    }

    let vendor = sqlx::query_as::<_, VendorRow>(
        "SELECT id, username, email FROM users WHERE role = 'vendor' LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    match vendor {
        Some(v) => {
            println!("\nVendor user: {} ({})", v.username, v.email);
            let bookstore = &bookstores[0];
            sqlx::query("UPDATE bookstores SET owner_id = $1 WHERE id = $2")
                .bind(v.id)
                .bind(bookstore.id)
                .execute(&pool)
                .await
                .unwrap();
            println!(" Linked '{}' to bookstore '{}'", v.username, bookstore.name);
        }
        None => println!(" No vendor user found"),
    }
}
