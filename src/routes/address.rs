use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    middleware::auth::AuthUser,
    models::address::{
        AddressListResponse, AddressResponse,
        CreateAddressRequest, UpdateAddressRequest,
    },
};

pub async fn get_addresses(
    state: State<AppState>,
    auth: AuthUser,
) -> Result<Json<AddressListResponse>, AppError> {
    crate::handlers::address::get_addresses(state, auth).await
}

pub async fn create_address(
    state: State<AppState>,
    auth: AuthUser,
    body: Json<CreateAddressRequest>,
) -> Result<Json<AddressResponse>, AppError> {
    crate::handlers::address::create_address(state, auth, body).await
}

pub async fn update_address(
    state: State<AppState>,
    auth: AuthUser,
    path: Path<Uuid>,
    body: Json<UpdateAddressRequest>,
) -> Result<Json<AddressResponse>, AppError> {
    crate::handlers::address::update_address(state, auth, path, body).await
}

pub async fn delete_address(
    state: State<AppState>,
    auth: AuthUser,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::handlers::address::delete_address(state, auth, path).await
}

pub async fn set_default_address(
    state: State<AppState>,
    auth: AuthUser,
    path: Path<Uuid>,
) -> Result<Json<AddressResponse>, AppError> {
    crate::handlers::address::set_default_address(state, auth, path).await
}