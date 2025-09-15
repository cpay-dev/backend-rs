use crate::{
  grpc::insert_api_key,
  http::{AppState, auth::ApiKey, chain_map::chain_code_str_from_proto_value, error::ApiHttpError, types::ChainDto},
};
use axum::{
  Json,
  extract::{Extension, State},
  response::IntoResponse,
};
use cpay_proto::cpay;

pub async fn list_chains(
  State(state): State<AppState>,
  Extension(ApiKey(api_key)): Extension<ApiKey>,
) -> Result<impl IntoResponse, ApiHttpError> {
  let mut client = state.grpc.merchant.clone();

  let req = cpay::api::v1::merchant::ListChainsRequest {};
  let req = tonic::Request::new(req);
  let req = insert_api_key(req, &api_key);

  let resp = client
    .list_chains(req)
    .await
    .map_err(|e| ApiHttpError::Internal(e.to_string()))?
    .into_inner();

  let mut chains: Vec<ChainDto> = Vec::with_capacity(resp.chains.len());
  for c in resp.chains.into_iter() {
    let id = chain_code_str_from_proto_value(c.id)?;
    chains.push(ChainDto { id, name: c.name });
  }

  Ok(Json(chains))
}
