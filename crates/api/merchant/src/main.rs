mod config;
mod error;
mod grpc;
mod http;

use crate::config::Config;
use crate::error::AppError;
use crate::grpc::GrpcState;
use crate::http::{auth, handlers};
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{error, info, trace};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to start merchant API");
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

  let grpc_state = GrpcState::connect(&config.merchant_grpc_addr).await?;

  let app_state = http::AppState { grpc: grpc_state };

  let router: Router = Router::new()
    .route("/blockchain/chains", get(handlers::list_chains))
    .layer(TraceLayer::new_for_http())
    .layer(axum::middleware::from_fn(auth::auth_middleware))
    .with_state(app_state);

  let addr: SocketAddr = config.listen_addr.parse()?;
  info!(%addr, grpc_addr = %config.merchant_grpc_addr, "starting merchant HTTP API");

  let listener = tokio::net::TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;
  info!(%local_addr, grpc_addr = %config.merchant_grpc_addr, "merchant HTTP API started");

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
  info!("merchant HTTP API stopped");
  Ok(())
}

fn resolve_config_path_from_args() -> Result<std::path::PathBuf, AppError> {
  let args: Vec<String> = std::env::args().collect();
  let mut path = std::path::PathBuf::from("config.json");
  let mut i = 1;
  while i < args.len() {
    if args[i] == "-config" || args[i] == "--config" {
      if i + 1 >= args.len() {
        return Err(AppError::Config("-config requires a file path".into()));
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
