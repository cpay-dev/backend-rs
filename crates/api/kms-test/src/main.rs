mod config;
mod error;

use crate::config::Config;
use crate::error::AppError;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use cpay_proto::cpay::api::v1::kms::{
  UnwrapKeyRequest, WrapKeyRequest, key_management_service_client::KeyManagementServiceClient,
};
use lib_crypto::ZeroizingKey;
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

  trace!(addr = %config.kms_grpc_addr, "connecting to kms");
  let mut kms_client = KeyManagementServiceClient::connect(config.kms_grpc_addr).await?;

  let plaintext = "Something";

  let transit_key = ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(AppError::AeadGenerateKey)?);
  let transit_cipher = XChaCha20Poly1305::new(&transit_key);
  let encrypted_data =
    lib_crypto::encrypt_data(&transit_cipher, plaintext.as_bytes().to_vec()).map_err(AppError::Crypto)?;

  let wrap_resp = kms_client
    .wrap_key(WrapKeyRequest {
      data: encrypted_data,
      transit_key: transit_key.to_vec(),
    })
    .await?
    .into_inner();

  let wrapped_data = lib_crypto::decrypt_data(&transit_cipher, wrap_resp.encrypted_data).map_err(AppError::Crypto)?;

  info!(?plaintext, "wrapped data");

  let transit_key = ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(AppError::AeadGenerateKey)?);
  let transit_cipher = XChaCha20Poly1305::new(&transit_key);
  let encrypted_data = lib_crypto::encrypt_data(&transit_cipher, wrapped_data.to_vec()).map_err(AppError::Crypto)?;

  let unwrap_resp = kms_client
    .unwrap_key(UnwrapKeyRequest {
      version: wrap_resp.version,
      data: encrypted_data,
      transit_key: transit_key.to_vec(),
    })
    .await?
    .into_inner();

  let decrypted_data =
    lib_crypto::decrypt_data(&transit_cipher, unwrap_resp.decrypted_data).map_err(AppError::Crypto)?;

  info!("unwrapped data");

  if let Ok(s) = String::from_utf8(decrypted_data.to_vec()) {
    info!(plaintext=?s, "decrypted data");
  } else {
    error!("failed to decrypt data");
  }

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
