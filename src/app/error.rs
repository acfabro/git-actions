// use axum::{
//     http::StatusCode,
//     response::{IntoResponse, Response},
//     Json,
// };
// use serde_json::json;
// use thiserror::Error;
//
// /// Errors that can occur during webhook data extraction
// #[derive(Debug, Error)]
// pub enum WebhookDataError {
//     #[error("Missing event type in payload")]
//     MissingEventType,
//
//     #[error("Missing branch information in payload")]
//     MissingBranch,
// }
//
// /// Errors that can occur during webhook processing
// #[derive(Debug, Error)]
// pub enum WebhookError {
// }
//
// // impl IntoResponse for WebhookError {
// //     fn into_response(self) -> Response {
// //         let (status, error_message) = match self {
// //         };
// //
// //         let body = Json(json!({"error": error_message}));
// //
// //         (status, body).into_response()
// //     }
// // }
