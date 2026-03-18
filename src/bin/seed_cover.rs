/// cargo run --bin seed_covers
///
/// Fetches real book cover images from Open Library for every book in the
/// database that doesn't yet have a cover_url, then writes the URL back.
///
/// Open Library cover flow:
///   1. Search: GET https://openlibrary.org/search.json?title=...&author=...&limit=1
///   2. Extract `cover_i` (cover ID) from the first result
///   3. Build URL: https://covers.openlibrary.org/b/id/{cover_i}-L.jpg
///
/// Run once after the 004 migration. Safe to re-run — skips books that
/// already have a cover_url.

use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// ── Open Library search response shape ───────────────────────────

#[derive(Debug, Deserialize)]
struct OLSearchResponse {
    docs: Vec<OLDoc>,
}

#[derive(Debug, Deserialize)]
struct OLDoc {
    cover_i:      Option<i64>,
    title:        Option<String>,
    #[serde(default)]
    author_name:  Vec<String>,
}

// ── Helpers ───────────────────────────────────────────────────────

fn cover_url_from_id(cover_i: i64) -> String {
    format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_i)
}

async fn fetch_cover(client: &Client, title: &str, author: &str) -> Option<String> {
    let url = format!(
        "https://openlibrary.org/search.json?title={}&author={}&limit=3&fields=cover_i,title,author_name",
        urlencoding::encode(title),
        urlencoding::encode(author),
    );

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    let body: OLSearchResponse = resp.json().await.ok()?;

    // Try to find a doc that has a cover_i
    // Prefer exact title match, fall back to first result with a cover
    let best = body.docs.iter().find(|d| {
        d.cover_i.is_some() &&
        d.title.as_deref()
            .map(|t| t.to_lowercase().contains(&title.to_lowercase()))
            .unwrap_or(false)
    }).or_else(|| body.docs.iter().find(|d| d.cover_i.is_some()))?;

    Some(cover_url_from_id(best.cover_i?))
}

// ── Main ──────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Connected to database");

    // Fetch all books that don't yet have a cover_url
    let books = sqlx::query_as::<_, (uuid::Uuid, String, String)>(
        "SELECT id, title, author FROM books WHERE cover_url IS NULL ORDER BY title"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch books");

    println!("📚 Found {} books without cover URLs\n", books.len());

    let client = Client::builder()
        .user_agent("Readdeck/1.0 (book delivery app; contact@readdeck.com)")
        .build()
        .expect("Failed to build HTTP client");

    let mut found     = 0;
    let mut not_found = 0;

    for (id, title, author) in &books {
        print!("  🔍 {} — {} ... ", title, author);

        match fetch_cover(&client, title, author).await {
            Some(url) => {
                sqlx::query("UPDATE books SET cover_url = $1 WHERE id = $2")
                    .bind(&url)
                    .bind(id)
                    .execute(&pool)
                    .await
                    .expect("Failed to update cover_url");

                println!("✅ {}", url);
                found += 1;
            }
            None => {
                println!("❌ not found on Open Library");
                not_found += 1;
            }
        }

        sleep(Duration::from_millis(300)).await;
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Covers found:     {}", found);
    println!("❌ Not found:        {}", not_found);
    println!("📚 Total processed:  {}", books.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nDone! Run `cargo run` to start the server with cover images.");
}