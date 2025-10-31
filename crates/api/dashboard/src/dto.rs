use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
	pub code: String,
	pub message: Option<String>,
}

#[derive(Serialize)]
pub struct ContinueAuthResponseDto {
	pub data: Option<ContinueAuthResponseDataDto>,
}

#[derive(Serialize)]
pub struct ContinueAuthResponseDataDto {
	pub provider_data: Option<ProviderCallbackDataDto>,
}

#[derive(Serialize)]
pub struct ProviderCallbackDataDto {
	pub email: String,
	pub email_verified: bool,
}

pub enum ApiError {
	Upstream {
		reason: &'static str,
		status: tonic::Status,
	},
	Internal(anyhow::Error),
	BadRequest(String),
}

impl ApiError {
	pub fn upstream(reason: &'static str, status: tonic::Status) -> Self {
		Self::Upstream { reason, status }
	}

	pub fn bad_request(message: String) -> Self {
		Self::BadRequest(message)
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		match self {
			Self::Upstream { reason, status } => {
				tracing::error!(reason, ?status, "upstream error");
				(StatusCode::BAD_GATEWAY, "").into_response()
			}
			Self::Internal(error) => {
				tracing::error!(?error, "internal error");
				(StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
			}
			Self::BadRequest(message) => {
				(StatusCode::BAD_REQUEST, message).into_response()
			}
		}
	}
}
