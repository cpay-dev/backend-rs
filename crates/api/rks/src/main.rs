mod config;
mod crypto;
mod error;
mod input;
mod kdf;
mod service;

use crate::config::Config;
use crate::crypto::WrappedMasterKey;
use crate::error::AppError;
use crate::input::{read_line, read_secure};
use crate::kdf::derive_master_key;
use crate::service::RootKeyService;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{error, info, trace};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to start rks api");
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

  let wrapper = WrappedMasterKey::new({
    let secret = read_secure("argon secret: ")?;
    let salt_ulid = {
      if config.salt.is_empty() {
        let salt_ulid_input = read_line("salt ulid: ")?;
        ulid::Ulid::from_string(&salt_ulid_input)?
      } else {
        ulid::Ulid::from_string(&config.salt)?
      }
    };
    let salt_bytes = zeroize::Zeroizing::new(salt_ulid.to_bytes());
    let password = read_secure("password: ")?;

    info!("deriving master key");
    let mk = derive_master_key(&config.argon, &secret, &password, &*salt_bytes)?;
    info!("key derived");
    mk
  })?;

  let service = RootKeyService::new(wrapper, config.version);

  let addr: SocketAddr = config.listen_addr.parse()?;
  info!(%addr, "starting gRPC");

  let listener = tokio::net::TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;
  info!(%local_addr, "serving gRPC");

  Server::builder()
    .add_service(service.into_server())
    .serve_with_incoming_shutdown(
      tokio_stream::wrappers::TcpListenerStream::new(listener),
      shutdown_signal(),
    )
    .await?;

  info!("rks gRPC API stopped");
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
