use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    routing::{get, post, put},
    Router,
};

mod models;
mod handlers;

use handlers::{AppState, create_tracking, get_tracking, mark_abnormal};

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/api/tracking", post(create_tracking))
        .route("/api/tracking/:waybill_no", get(get_tracking))
        .route("/api/tracking/:waybill_no/events/:event_id/abnormal", put(mark_abnormal))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("运单轨迹服务已启动，监听地址: 127.0.0.1:3000");
    println!("POST /api/tracking - 新增运单轨迹");
    println!("GET  /api/tracking/:waybill_no - 根据运单号查询轨迹（含异常检测）");
    println!("PUT  /api/tracking/:waybill_no/events/:event_id/abnormal - 手动标记轨迹异常");

    axum::serve(listener, app).await.unwrap();
}
