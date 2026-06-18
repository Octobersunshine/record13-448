use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub id: Uuid,
    pub waybill_no: String,
    pub status: String,
    pub location: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrackingRequest {
    pub waybill_no: String,
    pub status: String,
    pub location: String,
    pub description: String,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        ApiResponse {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}
