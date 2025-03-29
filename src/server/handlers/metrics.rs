use axum::{response::IntoResponse, Json};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

// Global atomic counters for metrics
static EVENTS_RECEIVED: AtomicUsize = AtomicUsize::new(0);
static RULES_MATCHED: AtomicUsize = AtomicUsize::new(0);
static ACTIONS_EXECUTED: AtomicUsize = AtomicUsize::new(0);
static ACTION_ERRORS: AtomicUsize = AtomicUsize::new(0);

/// Metrics endpoint handler
///
/// Returns metrics about the server's operation
pub async fn metrics() -> impl IntoResponse {
    Json(json!({
        "events_received": EVENTS_RECEIVED.load(Ordering::Relaxed),
        "rules_matched": RULES_MATCHED.load(Ordering::Relaxed),
        "actions_executed": ACTIONS_EXECUTED.load(Ordering::Relaxed),
        "action_errors": ACTION_ERRORS.load(Ordering::Relaxed),
    }))
}

/// Increment the events received counter
pub fn increment_events_received() {
    EVENTS_RECEIVED.fetch_add(1, Ordering::Relaxed);
}

/// Increment the rules matched counter
pub fn increment_rules_matched(count: usize) {
    RULES_MATCHED.fetch_add(count, Ordering::Relaxed);
}

/// Increment the actions executed counter
pub fn increment_actions_executed(count: usize) {
    ACTIONS_EXECUTED.fetch_add(count, Ordering::Relaxed);
}

/// Increment the action errors counter
pub fn increment_action_errors() {
    ACTION_ERRORS.fetch_add(1, Ordering::Relaxed);
}
