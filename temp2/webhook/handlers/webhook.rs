use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Error;

use crate::server::AppState;
use crate::webhook::error::WebhookError;

/// Generic webhook handler for all webhook types
#[axum::debug_handler]
pub async fn webhook_handler(
    State(state): State<AppState>,
    uri: Uri,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> anyhow::Result<impl IntoResponse> {
    // Get the path from the request URI
    let path = uri.path().to_string();
    
    // Convert HeaderMap to HashMap for easier handling
    let headers_map = headers_to_hashmap(&headers);
    
    tracing::info!("Received webhook request for path: {}", path);
    
    // Process the webhook using the webhook service
    state.webhook_service.process_webhook(&path, &body, &headers_map).await?;
    
    // Increment metrics
    // metrics::increment_events_received();
    
    StatusCode::OK
}

/// Convert HeaderMap to HashMap
fn headers_to_hashmap(headers: &axum::http::HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            map.insert(key.as_str().to_string(), v.to_string());
        }
    }
    map
}