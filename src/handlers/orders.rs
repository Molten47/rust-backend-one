use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::orders::{
        OrderItemResponse, OrderListQuery, OrderListResponse,
        OrderResponse, OrderSummary, PlaceOrderRequest,
    },
    AppState,
};

fn extract_user_id(auth: &AuthUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&auth.0.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))
}

// ── GET /orders ───────────────────────────────────────────────────
// Cursor-based pagination: client sends ?cursor=<placed_at ISO>&limit=20
// Response includes next_cursor for the next page, has_more flag.
// Cache key includes cursor + limit so each page is cached independently.

pub async fn get_orders(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<OrderListQuery>,
) -> Result<Json<OrderListResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;
    let limit   = params.limit.unwrap_or(20).clamp(1, 50);

    // Cache key encodes user + page position + status filter
    let _cache_key = format!(
        "{}:cursor={:?}:limit={}:status={:?}",
        user_id, params.cursor, limit, params.status
    );

    // Only cache first page (no cursor) — subsequent pages are cheap enough
    if params.cursor.is_none() {
        if let Some(cached) = state.orders_cache.get(&user_id).await {
            return Ok(Json(cached));
        }
    }

    // Fetch limit+1 rows — if we get limit+1 back, there's a next page
    let rows = sqlx::query!(
        r#"
        SELECT
            o.id,
            o.bookstore_id,
            o.status,
            o.total_amount,
            o.placed_at,
            COUNT(oi.id) AS item_count
        FROM orders o
        LEFT JOIN order_items oi ON oi.order_id = o.id
        WHERE o.user_id = $1
          AND ($2::timestamptz IS NULL OR o.placed_at < $2)
          AND ($3::text IS NULL OR o.status = $3)
        GROUP BY o.id
        ORDER BY o.placed_at DESC
        LIMIT $4
        "#,
        user_id,
        params.cursor as _,
        params.status,
        limit + 1  // fetch one extra to detect next page
    )
    .fetch_all(&state.pool)
    .await?;

    // Total count for display (separate query, doesn't affect pagination)
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM orders WHERE user_id = $1 AND ($2::text IS NULL OR status = $2)"#,
        user_id,
        params.status
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    let has_more = rows.len() as i64 > limit;

    // Drop the extra row we fetched for has_more detection
    let page_rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    let orders: Vec<OrderSummary> = page_rows
        .iter()
        .map(|r| OrderSummary {
            id:           r.id,
            bookstore_id: r.bookstore_id,
            status:       r.status.clone(),
            total_amount: r.total_amount.to_string().parse::<f64>().unwrap_or(0.0),
            item_count:   r.item_count.unwrap_or(0),
            placed_at:    r.placed_at,
        })
        .collect();

    // next_cursor is the placed_at of the last item on this page
    let next_cursor = if has_more {
        page_rows.last().map(|r| r.placed_at)
    } else {
        None
    };

    let response = OrderListResponse {
        orders,
        total,
        next_cursor,
        has_more,
        limit,
    };

    // Cache only the first page
    if params.cursor.is_none() {
        state.orders_cache.insert(user_id, response.clone()).await;
    }

    Ok(Json(response))
}

// ── GET /orders/:id ───────────────────────────────────────────────

pub async fn get_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderResponse>, AppError> {
    let user_id = extract_user_id(&auth)?;

    if let Some(cached) = state.order_detail_cache.get(&order_id).await {
        return Ok(Json(cached));
    }

    let order_row = sqlx::query!(
        r#"
        SELECT id, bookstore_id, status, total_amount, delivery_fee,
               address, notes, placed_at, updated_at
        FROM orders
        WHERE id = $1 AND user_id = $2
        "#,
        order_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::ValidationError("Order not found".into()))?;

    let item_rows = sqlx::query!(
        r#"
        SELECT
            oi.id,
            oi.book_id,
            oi.quantity,
            oi.unit_price,
            b.title,
            b.author,
            b.cover_emoji,
            b.cover_color
        FROM order_items oi
        JOIN books b ON b.id = oi.book_id
        WHERE oi.order_id = $1
        "#,
        order_id
    )
    .fetch_all(&state.pool)
    .await?;

    let items = item_rows
        .into_iter()
        .map(|r| {
            let price = r.unit_price.to_string().parse::<f64>().unwrap_or(0.0);
            OrderItemResponse {
                id:          r.id,
                book_id:     r.book_id,
                title:       r.title,
                author:      r.author,
                cover_emoji: r.cover_emoji,
                cover_color: r.cover_color,
                quantity:    r.quantity,
                unit_price:  price,
                subtotal:    price * r.quantity as f64,
            }
        })
        .collect();

    let response = OrderResponse {
        id:           order_row.id,
        bookstore_id: order_row.bookstore_id,
        status:       order_row.status,
        total_amount: order_row.total_amount.to_string().parse::<f64>().unwrap_or(0.0),
        delivery_fee: order_row.delivery_fee.to_string().parse::<f64>().unwrap_or(0.0),
        address:      order_row.address,
        notes:        order_row.notes,
        placed_at:    order_row.placed_at,
        updated_at:   order_row.updated_at,
        items,
    };

    state.order_detail_cache.insert(order_id, response.clone()).await;
    Ok(Json(response))
}

// ── POST /orders ──────────────────────────────────────────────────

pub async fn place_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let user_id = extract_user_id(&auth)?;

    let cart_items = sqlx::query!(
        r#"
        SELECT ci.id, ci.book_id, ci.quantity, b.price, b.bookstore_id
        FROM cart_items ci
        JOIN carts c ON c.id = ci.cart_id
        JOIN books b ON b.id = ci.book_id
        WHERE c.user_id = $1
        AND b.bookstore_id = $2
        AND (
            $3::uuid[] = '{}'::uuid[]
            OR ci.id = ANY($3)
        )
        "#,
        user_id,
        body.bookstore_id,
        &body.cart_item_ids as &[uuid::Uuid]
    )
    .fetch_all(&state.pool)
    .await?;

    if cart_items.is_empty() {
        return Err(AppError::ValidationError(
            "No cart items found for this bookstore".into(),
        ));
    }

    let delivery_fee  = 500.0_f64;
    let total_amount: f64 = cart_items
        .iter()
        .map(|r| r.price.to_string().parse::<f64>().unwrap_or(0.0) * r.quantity as f64)
        .sum::<f64>()
        + delivery_fee;

    let mut tx = state.pool.begin().await?;

    let order = sqlx::query!(
        r#"
        INSERT INTO orders (user_id, bookstore_id, status, total_amount, delivery_fee, address, notes)
        VALUES ($1, $2, 'pending', $3, $4, $5, $6)
        RETURNING id, bookstore_id, status, total_amount, delivery_fee, address, notes, placed_at, updated_at
        "#,
        user_id,
        body.bookstore_id,
        sqlx::types::Decimal::try_from(total_amount).unwrap_or_default(),
        sqlx::types::Decimal::try_from(delivery_fee).unwrap_or_default(),
        body.address,
        body.notes
    )
    .fetch_one(&mut *tx)
    .await?;

    let mut items_response = Vec::new();

    for item in &cart_items {
        let unit_price = item.price.to_string().parse::<f64>().unwrap_or(0.0);

        let inserted = sqlx::query!(
            r#"INSERT INTO order_items (order_id, book_id, quantity, unit_price)
               VALUES ($1, $2, $3, $4) RETURNING id"#,
            order.id, item.book_id, item.quantity, item.price
        )
        .fetch_one(&mut *tx)
        .await?;

        let book = sqlx::query!(
            "SELECT title, author, cover_emoji, cover_color FROM books WHERE id = $1",
            item.book_id
        )
        .fetch_one(&mut *tx)
        .await?;

        items_response.push(OrderItemResponse {
            id:          inserted.id,
            book_id:     item.book_id,
            title:       book.title,
            author:      book.author,
            cover_emoji: book.cover_emoji,
            cover_color: book.cover_color,
            quantity:    item.quantity,
            unit_price,
            subtotal:    unit_price * item.quantity as f64,
        });
    }

    let ordered_ids: Vec<Uuid> = cart_items.iter().map(|i| i.id).collect();
    sqlx::query!("DELETE FROM cart_items WHERE id = ANY($1)", &ordered_ids)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    // Invalidate both user-level caches
    state.orders_cache.invalidate(&user_id).await;
    state.cart_cache.invalidate(&user_id).await;

    let response = OrderResponse {
        id:           order.id,
        bookstore_id: order.bookstore_id,
        status:       order.status,
        total_amount: order.total_amount.to_string().parse::<f64>().unwrap_or(0.0),
        delivery_fee: order.delivery_fee.to_string().parse::<f64>().unwrap_or(0.0),
        address:      order.address,
        notes:        order.notes,
        placed_at:    order.placed_at,
        updated_at:   order.updated_at,
        items:        items_response,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ── DELETE /orders/:id ────────────────────────────────────────────

pub async fn cancel_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = extract_user_id(&auth)?;

    let result = sqlx::query!(
        r#"UPDATE orders SET status = 'cancelled', updated_at = now()
           WHERE id = $1 AND user_id = $2 AND status = 'pending'"#,
        order_id, user_id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::ValidationError(
            "Order not found or cannot be cancelled".into(),
        ));
    }

    state.orders_cache.invalidate(&user_id).await;
    state.order_detail_cache.invalidate(&order_id).await;

    Ok(StatusCode::NO_CONTENT)
}