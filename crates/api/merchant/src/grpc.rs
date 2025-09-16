use std::str::FromStr;

use crate::error::AppError;
use cpay_proto::cpay;
use tonic::metadata::{MetadataKey, MetadataValue};
use tonic::metadata::errors::InvalidMetadataValue;

const API_KEY_HEADER: &str = "x-api-key";

#[derive(Clone)]
pub struct GrpcState {
  pub merchant: cpay::api::v1::merchant::merchant_service_client::MerchantServiceClient<tonic::transport::Channel>,
}

impl GrpcState {
  pub async fn connect(addr: &str) -> Result<Self, AppError> {
    let merchant =
      cpay::api::v1::merchant::merchant_service_client::MerchantServiceClient::connect(addr.to_string()).await?;
    Ok(Self { merchant })
  }
}

pub fn insert_api_key<T>(mut req: tonic::Request<T>, api_key: &str) -> Result<tonic::Request<T>, InvalidMetadataValue> {
  let key = MetadataKey::from_static(API_KEY_HEADER);
  let val = MetadataValue::from_str(api_key)?;
  req.metadata_mut().insert(key, val);
  Ok(req)
}
