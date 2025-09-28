mod config;
mod error;
mod repo;

use crate::config::Config;
use crate::error::AppError;
use crate::repo::Repository;
use app_config::resolve_config_from_args;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use cpay_proto::cpay::api::v1::kms::{WrapKeyRequest, key_management_service_client::KeyManagementServiceClient};
use lib_crypto::ZeroizingKey;
use tracing::{debug, error, info, trace};

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

  let config: Config = resolve_config_from_args()?;
  debug!("config loaded");

  trace!(addr = %config.root_grpc_addr, "connecting to root key service");
  let mut rks_client = KeyManagementServiceClient::connect(config.root_grpc_addr).await?;

  let repo = Repository::init(&config.database).await?;
  info!("initializing kek...");

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
  info!("saving kek...");

  repo
    .insert_key(&crate::repo::KekRecord {
      key_version: i32::try_from(config.version).expect("version too large"),
      root_key_version: i32::try_from(root_key_version).expect("root key version too large"),
      status: crate::repo::KeyStatus::Active,
      encrypted_key: wrapped_key.to_vec(),
    })
    .await?;
  info!("kek saved");

  repo.shutdown().await?;
  Ok(())
}
