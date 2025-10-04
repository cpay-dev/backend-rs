mod config;
mod error;
mod service;

use crate::config::Config;
use crate::error::AppError;
use app_config::resolve_config_from_args;
use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, aead::Aead};
use cpay_proto::cpay::api::v1::kms::{
  UnwrapKeyRequest, WrapKeyRequest, key_management_service_client::KeyManagementServiceClient,
};
use tracing::{debug, error, info, trace};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to test wallet");
    std::process::exit(1);
  }
}

async fn start() -> Result<(), AppError> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .with_target(false)
    .compact()
    .init();

  let config: Config = resolve_config_from_args()?;
  debug!("config loaded");

  trace!(addr = %config.kms_grpc_addr, "connecting to kms service");
  let mut kms_client = KeyManagementServiceClient::connect(config.kms_grpc_addr).await?;

  Ok(())
}
