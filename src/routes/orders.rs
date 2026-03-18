use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    handlers::orders as orders_handler,
    middleware::auth::AuthUser,
    models::orders::{OrderListQuery, OrderListResponse, OrderResponse, PlaceOrderRequest},
    AppState,
};

pub async fn get_orders(
    state: State<AppState>,
    auth: AuthUser,
    query: Query<OrderListQuery>,
) -> Result<Json<OrderListResponse>, AppError> {
    orders_handler::get_orders(state, auth, query).await
}

pub async fn get_order(
    state: State<AppState>,
    auth: AuthUser,
    path: Path<Uuid>,
) -> Result<Json<OrderResponse>, AppError> {
    orders_handler::get_order(state, auth, path).await
}

pub async fn place_order(
    state: State<AppState>,
    auth: AuthUser,
    body: Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    orders_handler::place_order(state, auth, body).await
}

pub async fn cancel_order(
    state: State<AppState>,
    auth: AuthUser,
    path: Path<Uuid>,
) -> Result<StatusCode, AppError> {
    orders_handler::cancel_order(state, auth, path).await
}