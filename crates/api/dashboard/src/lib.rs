use axum::http::HeaderValue;
use axum::{
	Router,
	routing::{get, post},
};
use cpay_proto::cpay::api::v1::authn::authn_service_client::AuthnServiceClient;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{CorsLayer, Vary};
use tower_http::trace::TraceLayer;

pub mod dto;
mod service;

#[derive(Clone)]
pub struct AppState {
	pub authn_client: AuthnServiceClient<tonic::transport::Channel>,
}

pub fn build_router(state: AppState) -> Router {
	Router::new()
		.route("/authn/google", post(service::post_authn_google))
		.route("/authn/callback/google", get(service::get_authn_callback_google))
		.with_state(state)
		.layer(CookieManagerLayer::new())
		.layer(TraceLayer::new_for_http())
		.layer(
			CorsLayer::new()
				.allow_origin([
					HeaderValue::from_static("http://localhost:3000"),
					HeaderValue::from_static("http://192.168.88.248:3000"),
					HeaderValue::from_static("https://cpay.wtf"),
				])
				.vary(Vary::default()),
		)
}
