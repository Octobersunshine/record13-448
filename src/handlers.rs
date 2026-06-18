use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{TrackingEvent, CreateTrackingRequest, ApiResponse};

pub type AppState = Arc<Mutex<HashMap<String, Vec<TrackingEvent>>>>;

pub async fn create_tracking(
    State(state): State<AppState>,
    Json(payload): Json<CreateTrackingRequest>,
) -> (StatusCode, Json<ApiResponse<TrackingEvent>>) {
    if payload.waybill_no.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, "运单号不能为空")),
        );
    }
    if payload.status.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, "状态不能为空")),
        );
    }

    let timestamp = payload.timestamp.unwrap_or_else(|| Utc::now());

    let event = TrackingEvent {
        id: Uuid::new_v4(),
        waybill_no: payload.waybill_no.clone(),
        status: payload.status,
        location: payload.location,
        description: payload.description,
        timestamp,
    };

    let mut store = state.lock().await;
    let events = store
        .entry(payload.waybill_no.clone())
        .or_insert_with(Vec::new);

    let pos = events
        .iter()
        .position(|e| e.timestamp < event.timestamp)
        .unwrap_or(events.len());
    events.insert(pos, event.clone());

    (StatusCode::CREATED, Json(ApiResponse::success(event)))
}

pub async fn get_tracking(
    State(state): State<AppState>,
    Path(waybill_no): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<TrackingEvent>>>) {
    let store = state.lock().await;

    match store.get(&waybill_no) {
        Some(events) => {
            (StatusCode::OK, Json(ApiResponse::success(events.clone())))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(404, "未找到该运单的轨迹记录")),
        ),
    }
}
