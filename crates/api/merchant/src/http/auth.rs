use crate::http::error::ApiHttpError;
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use http::header::HeaderName;

const API_KEY_HEADER: &str = "x-api-key";

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, ApiHttpError> {
  let headers = req.headers();
  let api_key = extract_api_key(headers)?;
  req.extensions_mut().insert(ApiKey(api_key));
  Ok(next.run(req).await)
}

#[derive(Clone)]
pub struct ApiKey(pub String);

fn extract_api_key(headers: &HeaderMap) -> Result<String, ApiHttpError> {
  let name = HeaderName::from_static(API_KEY_HEADER);
  let val = headers.get(&name).ok_or(ApiHttpError::Unauthorized)?;
  let s = val.to_str().map_err(|_| ApiHttpError::Unauthorized)?.to_string();
  Ok(s)
}
