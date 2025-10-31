use std::time::Duration;

use axum::http::header::HeaderName;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{
	Router,
	routing::{get, post},
};
use cpay_proto::cpay::api::v1::authn::authn_service_client::AuthnServiceClient;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{AllowHeaders, CorsLayer, Vary};
use tower_http::trace::TraceLayer;

pub mod dto;
mod service;

#[derive(Clone)]
pub struct AppState {
	pub authn_client: AuthnServiceClient<tonic::transport::Channel>,
}

pub fn build_router(state: AppState) -> Router {
	// validate cache
	
	Router::new()
		.route("/authn/google", post(service::post_authn_google))
		.route("/authn/callback/google", get(service::get_authn_callback_google))
		.with_state(state)
		.layer(CookieManagerLayer::new())
		.layer(TraceLayer::new_for_http())
		.layer(
			CorsLayer::new()
				.allow_origin([
					HeaderValue::from_static("https://cpay.wtf"),
					HeaderValue::from_static("https://cpay.wtf:3443"),
				])
				.allow_methods([Method::GET, Method::POST])
				.allow_headers(AllowHeaders::list([
					AUTHORIZATION,
					CONTENT_TYPE,
					HeaderName::from_static("x-requested-with"),
				]))
				.allow_credentials(true)
				.vary(Vary::default())
				.max_age(Duration::from_secs(600)),
		)
}
