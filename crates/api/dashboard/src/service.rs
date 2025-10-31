use anyhow::{Context, anyhow};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE};
use axum::{
	Json,
	extract::{Query, State},
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tower_cookies::cookie::{Cookie, SameSite, time::Duration};

use crate::AppState;
use crate::dto::{
	ApiError, ContinueAuthResponseDataDto, ContinueAuthResponseDto, InitAuthResponseDto, ProviderCallbackDataDto,
};

use cpay_proto::cpay::api::v1::authn::{
	AuthProvider, ContinueAuthRequest, ContinueAuthResponse, InitAuthRequest, ProviderCallbackMethod, ProviderMethod,
	continue_auth_request, continue_auth_response, init_auth_request, init_auth_response::Continuation,
};

const GOOGLE_OAUTH_STATE_COOKIE: &str = "google_oauth_state";

pub async fn post_authn_google(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
	let mut client = state.authn_client.clone();

	let request = InitAuthRequest {
		method: Some(init_auth_request::Method::Provider(ProviderMethod {
			provider: AuthProvider::Google.into(),
		})),
	};

	let response = client
		.init_auth(request)
		.await
		.map_err(|status| ApiError::upstream("failed to init auth", status))?;

	let inner = response.into_inner();
	let (state_value, redirect_url) = match inner.continuation {
		Some(Continuation::Provider(p)) => (p.state, p.redirect_url),
		_ => {
			return Err(ApiError::Internal(anyhow!("missing provider continuation")));
		}
	};

	let mut headers = HeaderMap::new();
	let cookie = Cookie::build((GOOGLE_OAUTH_STATE_COOKIE, state_value))
		.path("/authn")
		.http_only(true)
		.secure(true)
		.same_site(SameSite::Lax)
		.max_age(Duration::seconds(600))
		.build();
	let cookie_value = HeaderValue::from_str(&cookie.to_string())
		.with_context(|| "failed to build Set-Cookie header")
		.map_err(ApiError::Internal)?;
	headers.insert(SET_COOKIE, cookie_value);

	let dto = InitAuthResponseDto { redirect_url };
	Ok((StatusCode::OK, headers, Json(dto)))
}

#[derive(serde::Deserialize)]
pub struct ProviderCallbackQuery {
	pub state: String,
	pub code: String,
}

pub async fn get_authn_callback_google(
	State(state): State<AppState>,
	jar: CookieJar,
	Query(query): Query<ProviderCallbackQuery>,
) -> Result<Response, ApiError> {
	let cookie_state = jar.get(GOOGLE_OAUTH_STATE_COOKIE).map(|c| c.value().to_string());
	if cookie_state.as_ref() != Some(&query.state) {
		return Err(ApiError::bad_request("invalid state".to_string()));
	}

	let mut client = state.authn_client.clone();
	let request = ContinueAuthRequest {
		method: Some(continue_auth_request::Method::ProviderCallback(
			ProviderCallbackMethod {
				state: query.state,
				code: query.code,
			},
		)),
	};

	let response = client.continue_auth(request).await;

	match response {
		Ok(rsp) => {
			let dto = map_continue_auth_response(rsp.into_inner());
			Ok(Json(dto).into_response())
		}
		Err(status) => {
			use tonic::Code;
			match status.code() {
				Code::InvalidArgument => Ok(StatusCode::BAD_REQUEST.into_response()),
				Code::Unauthenticated => Ok(StatusCode::UNAUTHORIZED.into_response()),
				_ => Err(ApiError::upstream("failed to continue auth", status)),
			}
		}
	}
}

fn map_continue_auth_response(resp: ContinueAuthResponse) -> ContinueAuthResponseDto {
	let data = match resp.data {
		Some(continue_auth_response::Data::ProviderData(pd)) => Some(ContinueAuthResponseDataDto {
			provider_data: Some(ProviderCallbackDataDto {
				email: pd.email,
				email_verified: pd.email_verified,
			}),
		}),
		None => None,
	};

	ContinueAuthResponseDto { data }
}
