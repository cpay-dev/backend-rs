use crate::{
  grpc::insert_api_key,
  http::{AppState, auth::ApiKey, error::ApiHttpError, types::ChainDto},
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

  let resp = state
    .grpc
    .chain
    .list_chains(req)
    .await
    .map_err(|e| ApiHttpError::Internal(format!("list chains: {}", e)))?
    .into_inner();

  let mut chains: Vec<ChainDto> = Vec::with_capacity(resp.chains.len());
  for c in resp.chains.into_iter() {
    let chain = cpay::blockchain::v1::Chain::try_from(c.id)
      .map_err(|_| ApiHttpError::Internal(format!("invalid chain: {}", c.id)))?;
    let id = chain.try_into()?;
    chains.push(ChainDto { id, name: c.name });
  }

  Ok(Json(chains))
}

pub async fn list_assets(
  State(mut state): State<AppState>,
  Extension(ApiKey(api_key)): Extension<ApiKey>,
  Path(id): Path<super::chain_map::ChainCode>,
) -> Result<impl IntoResponse, ApiHttpError> {
  print!("chain code: {id:?}, {id}");
  let chain_id: cpay::blockchain::v1::Chain = super::chain_map::ChainCode::from(id).into();

  let req = cpay::api::v1::merchant::ListAssetsRequest {
    chain_id: chain_id.into(),
  };
  let req = tonic::Request::new(req);
  let req = insert_api_key(req, &api_key).map_err(|_| ApiHttpError::Unauthorized)?;

  let resp = state
    .grpc
    .asset
    .list_assets(req)
    .await
    .map_err(|e| ApiHttpError::Internal(format!("list assets: {}", e)))?
    .into_inner();

  let mut assets: Vec<super::types::AssetDto> = Vec::with_capacity(resp.assets.len());
  for a in resp.assets.into_iter() {
    let chain = cpay::blockchain::v1::Chain::try_from(a.chain)
      .map_err(|_| ApiHttpError::Internal(format!("invalid chain: {}", a.chain)))?;
    let metadata = a
      .metadata
      .ok_or_else(|| ApiHttpError::Internal("asset metadata missing".into()))?;
    assets.push(super::types::AssetDto {
      id: a.id,
      chain: chain.try_into()?,
      name: a.name,
      symbol: a.symbol,
      address: metadata.address,
      decimals: metadata.decimals,
    });
  }

  Ok(Json(assets))
}

pub async fn create_payment_intent(
  State(mut state): State<AppState>,
  Extension(ApiKey(api_key)): Extension<ApiKey>,
  Json(req): Json<super::types::CreatePaymentIntentRequest>,
) -> Result<impl IntoResponse, ApiHttpError> {
  if req.asset_id.trim().is_empty() {
    return Err(ApiHttpError::BadRequest("asset_id is required".into()));
  }

  let has_usd = req.amount_usd.is_some();
  let has_asset = req.amount_asset.is_some();

  if has_usd == has_asset {
    return Err(ApiHttpError::BadRequest(
      "provide exactly one of amount_usd or amount_asset".into(),
    ));
  }

  let amount_oneof = if let Some(v) = req.amount_usd {
    let s = v.to_string_normalized();
    Some(cpay::api::v1::merchant::create_payment_intent_request::Amount::AmountUsd(s))
  } else if let Some(v) = req.amount_asset {
    let s = v.to_string_normalized();
    Some(cpay::api::v1::merchant::create_payment_intent_request::Amount::AmountAsset(s))
  } else {
    return Err(ApiHttpError::BadRequest("provide amount_usd or amount_asset".into()));
  };

  let grpc_req = cpay::api::v1::merchant::CreatePaymentIntentRequest {
    asset_id: req.asset_id,
    amount: amount_oneof,
  };
  let grpc_req = tonic::Request::new(grpc_req);
  let grpc_req = insert_api_key(grpc_req, &api_key).map_err(|_| ApiHttpError::Unauthorized)?;

  let resp = state
    .grpc
    .payment
    .create_payment_intent(grpc_req)
    .await
    .map_err(|e| ApiHttpError::Internal(format!("create payment intent: {}", e)))?
    .into_inner();

  let payment_intent = resp
    .payment_intent
    .ok_or_else(|| ApiHttpError::Internal("payment intent missing".into()))?;

  Ok(Json(super::types::PaymentIntentDto {
    id: payment_intent.id,
    status: super::types::PaymentIntentStatus::AwaitingPayment,
  }))
}
