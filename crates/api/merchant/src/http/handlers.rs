use crate::{
  grpc::insert_api_key,
  http::{AppState, auth::ApiKey, chain_map::chain_code_str_from_proto_value, error::ApiHttpError, types::ChainDto},
};
use axum::{
  Json,
  extract::{Extension, Path, State},
  response::IntoResponse,
};
use cpay_proto::cpay;

pub async fn list_chains(
  State(mut state): State<AppState>,
  Extension(ApiKey(api_key)): Extension<ApiKey>,
) -> Result<impl IntoResponse, ApiHttpError> {
  let req = cpay::api::v1::merchant::ListChainsRequest {};
  let req = tonic::Request::new(req);
  let req = insert_api_key(req, &api_key).map_err(|_| ApiHttpError::Unauthorized)?;

  let resp = state.grpc.merchant
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

pub async fn list_assets(
  State(mut state): State<AppState>,
  Extension(ApiKey(api_key)): Extension<ApiKey>,
  Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiHttpError> {
  let chain_id = super::chain_map::proto_enum_from_code_str(&id)?;

  let req = cpay::api::v1::merchant::ListAssetsRequest { chain: chain_id as i32 };
  let req = tonic::Request::new(req);
  let req = insert_api_key(req, &api_key).map_err(|_| ApiHttpError::Unauthorized)?;

  let resp = state.grpc.merchant
    .list_assets(req)
    .await
    .map_err(|e| ApiHttpError::Internal(e.to_string()))?
    .into_inner();

  let mut assets: Vec<super::types::AssetDto> = Vec::with_capacity(resp.assets.len());
  for a in resp.assets.into_iter() {
    let chain = super::chain_map::chain_code_str_from_proto_value(a.chain)?;
    let metadata = a.metadata.ok_or_else(|| ApiHttpError::Internal("asset metadata missing".into()))?;
    assets.push(super::types::AssetDto {
      id: a.id,
      chain,
      name: a.name,
      symbol: a.symbol,
      address: metadata.address,
      decimals: metadata.decimals,
    });
  }

  Ok(Json(assets))
}
