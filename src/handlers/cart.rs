use uuid::Uuid;
use sqlx::types::Decimal;

use crate::{
    AppState,
    errors::AppError,
    models::cart::{
        AddToCartRequest, UpdateQuantityRequest, AddToWishlistRequest,
        CartResponse, CartItemResponse, WishlistResponse, WishlistItemResponse,
    },
};

// ── HELPERS ───────────────────────────────────────────────────────

// Get or create a cart for the user
async fn get_or_create_cart(state: &AppState, user_id: Uuid) -> Result<Uuid, AppError> {
    // Try to find existing active cart
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM carts WHERE user_id = $1 AND status = 'active'"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(cart_id) = existing {
        return Ok(cart_id);
    }

    // No active cart — create one
    let cart_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO carts (user_id, status) VALUES ($1, 'active') RETURNING id"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(cart_id)
}

// Build a CartResponse from the DB — used after every write
async fn build_cart_response(state: &AppState, cart_id: Uuid) -> Result<CartResponse, AppError> {
    // Get cart status
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM carts WHERE id = $1"
    )
    .bind(cart_id)
    .fetch_one(&state.pool)
    .await?;

    // Get all items joined with book details
    let rows = sqlx::query!(
        r#"
        SELECT
            ci.id,
            ci.book_id,
            ci.quantity,
            ci.price_snapshot,
            b.title,
            b.author,
            b.cover_emoji,
            b.cover_color,
            b.cover_url,
            b.bookstore_id,
            bs.name AS bookstore_name
        FROM cart_items ci
        JOIN books b ON ci.book_id = b.id
        JOIN bookstores bs ON b.bookstore_id = bs.id
        WHERE ci.cart_id = $1
        ORDER BY ci.created_at ASC  
        "#,
        cart_id
    )
    .fetch_all(&state.pool)
    .await?;

    let items: Vec<CartItemResponse> = rows.iter().map(|r| {
        let price = r.price_snapshot.to_string().parse::<f64>().unwrap_or(0.0);
        let subtotal = price * r.quantity as f64;
     CartItemResponse {
        id:             r.id,
        book_id:        r.book_id,
        title:          r.title.clone(),
        author:         r.author.clone(),
        price_snapshot: price,
        quantity:       r.quantity,
        subtotal,
        bookstore_id:   r.bookstore_id,
        bookstore_name: r.bookstore_name.clone(),
        cover_emoji:    r.cover_emoji.clone(),
        cover_color:    r.cover_color.clone(),
        cover_url:      r.cover_url.clone(),
}
    }).collect();

    let total = items.iter().map(|i| i.subtotal).sum();
    let item_count = items.iter().map(|i| i.quantity).sum();

    Ok(CartResponse {
        id: cart_id,
        status,
        items,
        total,
        item_count,
    })
}

// ── CART HANDLERS ─────────────────────────────────────────────────

// GET /cart
pub async fn get_cart_handler(
    state: &AppState,
    user_id: Uuid,
) -> Result<axum::Json<CartResponse>, AppError> {
    // Check cache first
    if let Some(cached) = state.cart_cache.get(&user_id).await {
        return Ok(axum::Json(cached));
    }

    let cart_id = get_or_create_cart(state, user_id).await?;
    let response = build_cart_response(state, cart_id).await?;

    // Populate cache
    state.cart_cache.insert(user_id, response.clone()).await;

    Ok(axum::Json(response))
}

// POST /cart/items
pub async fn add_item_handler(
    state: &AppState,
    user_id: Uuid,
    body: AddToCartRequest,
) -> Result<axum::Json<CartResponse>, AppError> {
    if body.quantity < 1 {
        return Err(AppError::ValidationError("Quantity must be at least 1".into()));
    }

    let cart_id = get_or_create_cart(state, user_id).await?;

    // Fetch current book price for snapshot
    let price = sqlx::query_scalar::<_, Decimal>(
        "SELECT price FROM books WHERE id = $1 AND in_stock = true"
    )
    .bind(body.book_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Book not found or out of stock".into()))?;

    // Upsert — if book already in cart, increment quantity
    sqlx::query(
        r#"
        INSERT INTO cart_items (cart_id, book_id, quantity, price_snapshot)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (cart_id, book_id)
        DO UPDATE SET
            quantity   = cart_items.quantity + EXCLUDED.quantity,
            updated_at = NOW()
        "#
    )
    .bind(cart_id)
    .bind(body.book_id)
    .bind(body.quantity)
    .bind(price)
    .execute(&state.pool)
    .await?;

    // Invalidate cache then rebuild
    state.cart_cache.invalidate(&user_id).await;
    let response = build_cart_response(state, cart_id).await?;
    state.cart_cache.insert(user_id, response.clone()).await;

    Ok(axum::Json(response))
}

// PATCH /cart/items/:item_id
pub async fn update_quantity_handler(
    state: &AppState,
    user_id: Uuid,
    item_id: Uuid,
    body: UpdateQuantityRequest,
) -> Result<axum::Json<CartResponse>, AppError> {
    if body.quantity < 1 {
        return Err(AppError::ValidationError("Quantity must be at least 1".into()));
    }

    // Verify item belongs to this user's cart
    let cart_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT c.id FROM carts c
        JOIN cart_items ci ON ci.cart_id = c.id
        WHERE ci.id = $1 AND c.user_id = $2 AND c.status = 'active'
        "#
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Cart item not found".into()))?;

    sqlx::query(
        "UPDATE cart_items SET quantity = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(body.quantity)
    .bind(item_id)
    .execute(&state.pool)
    .await?;

    // Invalidate cache then rebuild
    state.cart_cache.invalidate(&user_id).await;
    let response = build_cart_response(state, cart_id).await?;
    state.cart_cache.insert(user_id, response.clone()).await;

    Ok(axum::Json(response))
}

// DELETE /cart/items/:item_id
pub async fn remove_item_handler(
    state: &AppState,
    user_id: Uuid,
    item_id: Uuid,
) -> Result<axum::Json<CartResponse>, AppError> {
    // Verify item belongs to this user's cart
    let cart_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT c.id FROM carts c
        JOIN cart_items ci ON ci.cart_id = c.id
        WHERE ci.id = $1 AND c.user_id = $2 AND c.status = 'active'
        "#
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Cart item not found".into()))?;

    sqlx::query("DELETE FROM cart_items WHERE id = $1")
        .bind(item_id)
        .execute(&state.pool)
        .await?;

    // Invalidate cache then rebuild
    state.cart_cache.invalidate(&user_id).await;
    let response = build_cart_response(state, cart_id).await?;
    state.cart_cache.insert(user_id, response.clone()).await;

    Ok(axum::Json(response))
}

// DELETE /cart
pub async fn clear_cart_handler(
    state: &AppState,
    user_id: Uuid,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let cart_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM carts WHERE user_id = $1 AND status = 'active'"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("No active cart found".into()))?;

    sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
        .bind(cart_id)
        .execute(&state.pool)
        .await?;

    // Invalidate cache
    state.cart_cache.invalidate(&user_id).await;

    Ok(axum::Json(serde_json::json!({
        "message": "Cart cleared successfully"
    })))
}

// ── WISHLIST HANDLERS ─────────────────────────────────────────────

// Get or create wishlist for user
async fn get_or_create_wishlist(state: &AppState, user_id: Uuid) -> Result<Uuid, AppError> {
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM wishlists WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(wl_id) = existing {
        return Ok(wl_id);
    }

    let wl_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO wishlists (user_id) VALUES ($1) RETURNING id"
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(wl_id)
}
async fn build_wishlist_response(
    state: &AppState,
    wishlist_id: Uuid,
) -> Result<WishlistResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            wi.id,
            wi.book_id,
            wi.added_at,
            b.title,
            b.author,
            b.price
        FROM wishlist_items wi
        JOIN books b ON wi.book_id = b.id
        WHERE wi.wishlist_id = $1
        ORDER BY wi.added_at DESC
        "#,
        wishlist_id
    )
    .fetch_all(&state.pool)
    .await?;

    let items = rows.iter().map(|r| WishlistItemResponse {
        id:       r.id,
        book_id:  r.book_id,
        title:    r.title.clone(),
        author:   r.author.clone(),
        price:    r.price.to_string().parse::<f64>().unwrap_or(0.0),
        added_at: r.added_at,
    }).collect();

    Ok(WishlistResponse { id: wishlist_id, items })
}

pub async fn get_wishlist_handler(
    state: &AppState,
    user_id: Uuid,
) -> Result<axum::Json<WishlistResponse>, AppError> {
    let wishlist_id = get_or_create_wishlist(state, user_id).await?;
    let response = build_wishlist_response(state, wishlist_id).await?;
    Ok(axum::Json(response))
}

pub async fn add_to_wishlist_handler(
    state: &AppState,
    user_id: Uuid,
    body: AddToWishlistRequest,
) -> Result<axum::Json<WishlistResponse>, AppError> {
    let wishlist_id = get_or_create_wishlist(state, user_id).await?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM books WHERE id = $1)"
    )
    .bind(body.book_id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::ValidationError("Book not found".into()));
    }

    sqlx::query(
        r#"
        INSERT INTO wishlist_items (wishlist_id, book_id)
        VALUES ($1, $2)
        ON CONFLICT (wishlist_id, book_id) DO NOTHING
        "#
    )
    .bind(wishlist_id)
    .bind(body.book_id)
    .execute(&state.pool)
    .await?;

    let response = build_wishlist_response(state, wishlist_id).await?;
    Ok(axum::Json(response))
}

pub async fn remove_from_wishlist_handler(
    state: &AppState,
    user_id: Uuid,
    item_id: Uuid,
) -> Result<axum::Json<WishlistResponse>, AppError> {
    let wishlist_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT w.id FROM wishlists w
        JOIN wishlist_items wi ON wi.wishlist_id = w.id
        WHERE wi.id = $1 AND w.user_id = $2
        "#
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::ValidationError("Wishlist item not found".into()))?;

    sqlx::query("DELETE FROM wishlist_items WHERE id = $1")
        .bind(item_id)
        .execute(&state.pool)
        .await?;

    let response = build_wishlist_response(state, wishlist_id).await?;
    Ok(axum::Json(response))
}

pub async fn move_to_cart_handler(
    state: &AppState,
    user_id: Uuid,
    item_id: Uuid,
) -> Result<axum::Json<CartResponse>, AppError> {
    let (wishlist_id, book_id) = sqlx::query!(
        r#"
        SELECT w.id as wishlist_id, wi.book_id
        FROM wishlists w
        JOIN wishlist_items wi ON wi.wishlist_id = w.id
        WHERE wi.id = $1 AND w.user_id = $2
        "#,
        item_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .map(|r| (r.wishlist_id, r.book_id))
    .ok_or(AppError::ValidationError("Wishlist item not found".into()))?;

    // Add to cart
    let add_request = AddToCartRequest { book_id, quantity: 1 };
    let _ = add_item_handler(state, user_id, add_request).await?;

    // Remove from wishlist
    sqlx::query("DELETE FROM wishlist_items WHERE id = $1")
        .bind(item_id)
        .execute(&state.pool)
        .await?;

    let _ = wishlist_id;

    // Return updated cart
    let cart_id = get_or_create_cart(state, user_id).await?;
    let response = build_cart_response(state, cart_id).await?;
    Ok(axum::Json(response))
}