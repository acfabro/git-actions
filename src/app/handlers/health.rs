use axum::{response::IntoResponse, Json};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check endpoint handler
#[axum::debug_handler]
pub async fn health_check() -> impl IntoResponse {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": uptime,
    }))
}
