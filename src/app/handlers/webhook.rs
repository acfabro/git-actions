use axum::extract::Path;
use axum::Json;
use axum::response::IntoResponse;
use tracing::debug;

pub async fn webhook(
    Path(path): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // TODO implement webhook handler

    debug!("webhook: path={}, payload={:?}", &path, &payload);

    // return a 200 OK response for now
    (axum::http::StatusCode::OK, format!("Webhook received: {path}"))
}
