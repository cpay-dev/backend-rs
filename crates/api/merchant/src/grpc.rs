use std::str::FromStr;

use crate::error::AppError;
use cpay_proto::cpay::api::v1::merchant::chain_service_client;
use cpay_proto::cpay::api::v1::merchant::asset_service_client;
use cpay_proto::cpay::api::v1::merchant::payment_intent_service_client;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::metadata::{MetadataKey, MetadataValue};

const API_KEY_HEADER: &str = "x-api-key";

#[derive(Clone)]
pub struct GrpcState {
  pub chain: chain_service_client::ChainServiceClient<tonic::transport::Channel>,
  pub asset: asset_service_client::AssetServiceClient<tonic::transport::Channel>,
  pub payment: payment_intent_service_client::PaymentIntentServiceClient<tonic::transport::Channel>,
}

impl GrpcState {
  pub async fn connect(addr: &str) -> Result<Self, AppError> {
    let conn = tonic::transport::Endpoint::new(addr.to_string())?.connect().await?;
    let chain = chain_service_client::ChainServiceClient::new(conn.clone());
    let asset = asset_service_client::AssetServiceClient::new(conn.clone());
    let payment = payment_intent_service_client::PaymentIntentServiceClient::new(conn.clone());
    Ok(Self { chain, asset, payment })
  }
}

pub fn insert_api_key<T>(mut req: tonic::Request<T>, api_key: &str) -> Result<tonic::Request<T>, InvalidMetadataValue> {
  let key = MetadataKey::from_static(API_KEY_HEADER);
  let val = MetadataValue::from_str(api_key)?;
  req.metadata_mut().insert(key, val);
  Ok(req)
}
