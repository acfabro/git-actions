use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::server::{
    error::{WebhookDataError, WebhookError},
    AppState,
};
use super::metrics; // Assuming metrics is in the parent handlers module

/// Bitbucket webhook handler
#[axum::debug_handler]
pub async fn bitbucket_webhook(
    State(state): State<AppState>,
    uri: Uri,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, WebhookError> {
    // Get the path from the request URI
    let path = uri.path().to_string();
    // Remove the "/webhook" prefix if present
    let path = path.strip_prefix("/webhook").unwrap_or(&path).to_string();

    tracing::info!("Processing Bitbucket webhook for path: {}", path);

    // Extract event details from the payload with error handling
    let event_type = get_event_type(&body)?;
    let branch = get_branch(&body)?;
    let paths = get_modified_paths(&body);

    tracing::info!("Event type: {}, Branch: {}", event_type, branch);

    // Find matching rules using the RuleMatcher from AppState
    let matching_rules = state
        .rule_matcher
        .find_matching_rules(&path, &event_type, &branch, &paths);

    tracing::info!("Found {} matching rules", matching_rules.len());

    // Execute actions for matching rules
    if let Err(e) = state.rule_matcher.execute_actions(matching_rules, &body) {
        tracing::error!("Error executing actions: {}", e);
        return Err(WebhookError::ActionExecutionError(e.to_string()));
    }

    // Increment metrics
    metrics::increment_events_received();

    Ok(StatusCode::OK)
}

// --- Bitbucket Extraction Helpers ---

fn get_event_type(payload: &Value) -> Result<String, WebhookDataError> {
    payload["eventKey"] // Corrected key based on common Bitbucket payloads
        .as_str()
        .map(|s| s.to_string())
        .ok_or(WebhookDataError::MissingEventType)
}

fn get_branch(payload: &Value) -> Result<String, WebhookDataError> {
    // Look in different possible locations based on event type
    if let Some(ref_change) = payload["refChanges"].as_array().and_then(|arr| arr.get(0)) {
        // Push event
        if let Some(branch) = ref_change["refId"].as_str() {
            return Ok(branch.replace("refs/heads/", ""));
        }
    } else if let Some(pr) = payload["pullRequest"].as_object() {
        // PR event
        if let Some(branch) = pr["fromRef"].as_object().and_then(|r| r["displayId"].as_str()) {
             return Ok(branch.to_string());
        }
    }
    // Add more extraction logic for different Bitbucket events if needed
    Err(WebhookDataError::MissingBranch)
}

fn get_modified_paths(payload: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    // Look in different possible locations based on event type
    if let Some(changes) = payload["changes"].as_array() { // Common in PR events
        for change in changes {
            if let Some(path_obj) = change["path"].as_object() {
                 if let Some(path_str) = path_obj["toString"].as_str() {
                     paths.push(path_str.to_string());
                 }
            }
        }
    } else if let Some(commits) = payload["commits"].as_array() { // Common in push events (via changesets)
         for commit in commits {
             if let Some(values) = commit["values"].as_array() {
                 for val in values {
                     if let Some(file_changes) = val["changes"].as_object() {
                         if let Some(file_values) = file_changes["values"].as_array() {
                             for file_change in file_values {
                                 if let Some(path_obj) = file_change["path"].as_object() {
                                     if let Some(path_str) = path_obj["toString"].as_str() {
                                         paths.push(path_str.to_string());
                                     }
                                 }
                             }
                         }
                     }
                 }
             }
         }
    }
    // Add more extraction logic if needed
    paths
}