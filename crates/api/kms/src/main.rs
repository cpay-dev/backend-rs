mod config;
mod error;
mod repo;
mod service;

use crate::config::Config;
use crate::error::AppError;
use crate::repo::Repository;
use crate::service::KmsService;
use app_config::resolve_config_from_args;
use cpay_proto::cpay::api::v1::kms::key_management_service_client::KeyManagementServiceClient;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{debug, error, info, trace};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to start kms api");
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

  let repo = Repository::init(&config.database).await?;
  let keys = repo.load_keys().await?;

  let active_key = keys
    .iter()
    .filter(|k| matches!(k.status, crate::repo::KeyStatus::Active))
    .max_by_key(|k| k.key_version)
    .cloned()
    .ok_or(AppError::NoActiveKek)?;

  trace!(addr = %config.rks_grpc_addr, "connecting to root key service");
  let rks_client = KeyManagementServiceClient::connect(config.rks_grpc_addr).await?;
  let service = KmsService::new(rks_client, keys, active_key);

  let addr: SocketAddr = config.listen_addr.parse()?;
  let listener = tokio::net::TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;
  info!(?local_addr, "serving gRPC");

  Server::builder()
    .add_service(service.into_server())
    .serve_with_incoming_shutdown(
      tokio_stream::wrappers::TcpListenerStream::new(listener),
      shutdown_signal(),
    )
    .await?;

  info!("kms gRPC API stopped");
  repo.shutdown().await?;
  Ok(())
}

async fn shutdown_signal() {
  #[cfg(unix)]
  {
    let mut term_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install SIGTERM handler");
    tokio::select! {
      _ = tokio::signal::ctrl_c() => {
        info!("received SIGINT; shutting down");
      }
      _ = term_signal.recv() => {
        info!("received SIGTERM; shutting down");
      }
    }
    return;
  }

  #[cfg(not(unix))]
  {
    let _ = tokio::signal::ctrl_c().await;
    info!("received interrupt; shutting down");
  }
}
