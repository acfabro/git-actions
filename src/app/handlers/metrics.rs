use axum::{response::IntoResponse, Json};
use serde_json::json;

/// Metrics endpoint handler
///
/// Returns metrics about the server's operation
pub async fn metrics() -> impl IntoResponse {
    Json(json!({
        "events_received": 0,
        "rules_matched": 0,
        "actions_executed": 0,
        "action_errors": 0,
    }))
}
