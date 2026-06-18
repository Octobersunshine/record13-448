use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::models::{
    TrackingEvent, CreateTrackingRequest, MarkAbnormalRequest,
    ApiResponse, WaybillTrackingResult,
};

pub type AppState = Arc<Mutex<HashMap<String, Vec<TrackingEvent>>>>;

const STAGNATION_THRESHOLD_HOURS: i64 = 48;

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
        is_abnormal: false,
        abnormal_reason: None,
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

pub async fn mark_abnormal(
    State(state): State<AppState>,
    Path((waybill_no, event_id)): Path<(String, Uuid)>,
    Json(payload): Json<MarkAbnormalRequest>,
) -> (StatusCode, Json<ApiResponse<TrackingEvent>>) {
    if payload.reason.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(400, "异常原因不能为空")),
        );
    }

    let mut store = state.lock().await;

    let events = match store.get_mut(&waybill_no) {
        Some(evts) => evts,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(404, "未找到该运单的轨迹记录")),
            );
        }
    };

    let event = match events.iter_mut().find(|e| e.id == event_id) {
        Some(evt) => evt,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(404, "未找到该轨迹节点")),
            );
        }
    };

    event.is_abnormal = true;
    event.abnormal_reason = Some(payload.reason);

    (StatusCode::OK, Json(ApiResponse::success(event.clone())))
}

fn check_stagnation(events: &[TrackingEvent]) -> (bool, Option<String>) {
    if events.is_empty() {
        return (false, None);
    }

    let latest_event = &events[0];
    let now = Utc::now();
    let stagnation_duration = now.signed_duration_since(latest_event.timestamp);

    if stagnation_duration > Duration::hours(STAGNATION_THRESHOLD_HOURS) {
        let hours = stagnation_duration.num_hours();
        (
            true,
            Some(format!("物流已停滞超过 {} 小时", hours)),
        )
    } else {
        (false, None)
    }
}

pub async fn get_tracking(
    State(state): State<AppState>,
    Path(waybill_no): Path<String>,
) -> (StatusCode, Json<ApiResponse<WaybillTrackingResult>>) {
    let mut store = state.lock().await;

    let events = match store.get_mut(&waybill_no) {
        Some(evts) => evts,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(404, "未找到该运单的轨迹记录")),
            );
        }
    };

    let (is_abnormal, abnormal_reason) = check_stagnation(events);

    if is_abnormal {
        if let Some(ref reason) = abnormal_reason {
            if let Some(latest) = events.first_mut() {
                if !latest.is_abnormal {
                    latest.is_abnormal = true;
                    latest.abnormal_reason = Some(reason.clone());
                }
            }
        }
    }

    let result = WaybillTrackingResult {
        waybill_no: waybill_no.clone(),
        is_abnormal,
        abnormal_reason,
        events: events.clone(),
    };

    (StatusCode::OK, Json(ApiResponse::success(result)))
}
