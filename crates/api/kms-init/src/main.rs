mod config;
mod error;
mod repo;

use crate::config::Config;
use crate::error::AppError;
use crate::repo::Repository;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use cpay_proto::cpay::api::v1::kms::{WrapKeyRequest, key_management_service_client::KeyManagementServiceClient};
use lib_crypto::ZeroizingKey;
use tracing::{error, info, trace};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to create KEK");
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

  trace!(addr = %config.root_grpc_addr, "connecting to root key service");
  let mut rks_client = KeyManagementServiceClient::connect(config.root_grpc_addr).await?;

  let repo = Repository::init(&config.database).await?;

  let (wrapped_key, root_key_version) = {
    let rnd_key = ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(AppError::AeadGenerateKey)?);
    let transit_key = ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(AppError::AeadGenerateKey)?);
    let transit_cipher = XChaCha20Poly1305::new(&transit_key);

    let transit_data = lib_crypto::encrypt_data(&transit_cipher, rnd_key.to_vec())?;

    let response = rks_client
      .wrap_key(WrapKeyRequest {
        data: transit_data,
        transit_key: transit_key.to_vec(),
      })
      .await
      .map_err(AppError::Grpc)?
      .into_inner();

    let wrapped_key = lib_crypto::decrypt_data(&transit_cipher, response.encrypted_data)?;
    (wrapped_key, response.version)
  };

  repo
    .insert_key(&crate::repo::KekRecord {
      key_version: i32::try_from(config.version).expect("version too large"),
      root_key_version: i32::try_from(root_key_version).expect("root key version too large"),
      status: crate::repo::KeyStatus::Active,
      encrypted_key: wrapped_key.to_vec(),
    })
    .await?;
  info!("key inserted");

  repo.shutdown().await?;
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
