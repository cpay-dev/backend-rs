use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Debug)]
pub enum ApiHttpError {
  Unauthorized,
  BadRequest(String),
  Internal(String),
}

impl IntoResponse for ApiHttpError {
  fn into_response(self) -> Response {
    match self {
      ApiHttpError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
      ApiHttpError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
      ApiHttpError::Internal(msg) => {
        error!(error = %msg, "internal error while handling request");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
      }
    }
  }
}
