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
use app_config::resolve_config_from_args;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{debug, error, info};

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

  let config: Config = resolve_config_from_args()?;
  debug!("config loaded");

  let wrapper = WrappedMasterKey::new({
    _ = read_line("reading input")?;

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

    use sha3::Digest;
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(&mk);
    let hash = hasher.finalize();
    let hash = to_hex(&hash);
    info!(hash = ?hash, "key derived");
    if hash != config.expected_hash {
      return Err(AppError::InvalidHash(hash));
    }
    mk
  })?;

  let service = RootKeyService::new(wrapper, config.version);

  let addr: SocketAddr = config.listen_addr.parse()?;
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

fn to_hex(bytes: &[u8]) -> String {
  const LUT: &[u8; 16] = b"0123456789abcdef";
  let mut v = Vec::with_capacity(bytes.len() * 2);
  for &b in bytes {
    v.push(LUT[(b >> 4) as usize]);
    v.push(LUT[(b & 0xF) as usize]);
  }
  // Safety: we only inserted valid ASCII
  unsafe { String::from_utf8_unchecked(v) }
}
