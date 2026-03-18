use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    middleware::auth::AuthUser,
    models::cart::{
        AddToCartRequest, UpdateQuantityRequest, AddToWishlistRequest,
        CartResponse, WishlistResponse,
    },
};

// GET /cart
pub async fn get_cart(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<CartResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::get_cart_handler(&state, user_id).await
}

// POST /cart/items
pub async fn add_item(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<AddToCartRequest>,
) -> Result<Json<CartResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::add_item_handler(&state, user_id, body).await
}

// PATCH /cart/items/:item_id
pub async fn update_quantity(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(item_id): Path<Uuid>,
    Json(body): Json<UpdateQuantityRequest>,
) -> Result<Json<CartResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::update_quantity_handler(&state, user_id, item_id, body).await
}

// DELETE /cart/items/:item_id
pub async fn remove_item(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<Json<CartResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::remove_item_handler(&state, user_id, item_id).await
}

// DELETE /cart
pub async fn clear_cart(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::clear_cart_handler(&state, user_id).await
}

// GET /wishlist
pub async fn get_wishlist(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<WishlistResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::get_wishlist_handler(&state, user_id).await
}

// POST /wishlist/items
pub async fn add_to_wishlist(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(body): Json<AddToWishlistRequest>,
) -> Result<Json<WishlistResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::add_to_wishlist_handler(&state, user_id, body).await
}

// DELETE /wishlist/items/:item_id
pub async fn remove_from_wishlist(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<Json<WishlistResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::remove_from_wishlist_handler(&state, user_id, item_id).await
}

// POST /wishlist/items/:item_id/move
pub async fn move_to_cart(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<Json<CartResponse>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::TokenError("Invalid user ID".into()))?;

    crate::handlers::cart::move_to_cart_handler(&state, user_id, item_id).await
}