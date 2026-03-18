use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    handlers::events as events_handler,
    models::events::{EventListResponse, EventQuery, EventResponse},
    AppState,
};

pub async fn get_events(
    state: State<AppState>,
    query: Query<EventQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    events_handler::get_events(state, query).await
}

pub async fn get_event(
    state: State<AppState>,
    path: Path<Uuid>,
) -> Result<Json<EventResponse>, AppError> {
    events_handler::get_event(state, path).await
}