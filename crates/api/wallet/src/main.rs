mod config;
mod error;
mod service;

use crate::config::Config;
use crate::error::AppError;
use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, aead::Aead};
use cpay_proto::cpay::api::v1::kms::{
  UnwrapKeyRequest, WrapKeyRequest, key_management_service_client::KeyManagementServiceClient,
};
use tracing::{error, info, trace};

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

  let config_path = resolve_config_path_from_args()?;
  trace!(?config_path, "reading config at path");
  let config = Config::from_file(config_path)?;

  trace!(addr = %config.kms_grpc_addr, "connecting to kms service");
  let mut kms_client = KeyManagementServiceClient::connect(config.kms_grpc_addr).await?;

  Ok(())
}

fn resolve_config_path_from_args() -> Result<std::path::PathBuf, AppError> {
  let args: Vec<String> = std::env::args().collect();
  let mut path = std::path::PathBuf::from("config.json");
  let mut i = 1;
  while i < args.len() {
    if args[i] == "-config" || args[i] == "--config" {
      if i + 1 >= args.len() {
        return Err(AppError::MissingConfigPath);
      }
      path = std::path::PathBuf::from(&args[i + 1]);
      i += 2;
      continue;
    }
    i += 1;
  }
  Ok(path)
}
