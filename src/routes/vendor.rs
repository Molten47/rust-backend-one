use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::{
    errors::AppError,
    handlers::vendor as vendor_handler,
    middleware::auth::AuthUser,
    models::vendor::{VendorApplicationRequest, VendorApplicationResponse},
    AppState,
};

pub async fn apply_as_vendor(
    state: State<AppState>,
    auth: AuthUser,
    body: Json<VendorApplicationRequest>,
) -> Result<(StatusCode, Json<VendorApplicationResponse>), AppError> {
    vendor_handler::apply_as_vendor(state, auth, body).await
}

pub async fn get_application_status(
    state: State<AppState>,
    auth: AuthUser,
) -> Result<Json<VendorApplicationResponse>, AppError> {
    vendor_handler::get_application_status(state, auth).await
}