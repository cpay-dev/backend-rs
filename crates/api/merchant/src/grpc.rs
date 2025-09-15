use crate::error::AppError;
use cpay_proto::cpay;
use tonic::metadata::{MetadataKey, MetadataValue};

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

pub fn insert_api_key<T>(mut req: tonic::Request<T>, api_key: &str) -> tonic::Request<T> {
  if let Ok(key) = MetadataKey::from_bytes(b"x-api-key") {
    if let Ok(val) = MetadataValue::try_from(api_key) {
      req.metadata_mut().insert(key, val);
    }
  }
  req
}
