mod config;
mod error;
mod grpc;
mod http;

use crate::config::Config;
use crate::error::AppError;
use crate::grpc::GrpcState;
use crate::http::{auth, handlers};
use app_config::resolve_config_from_args;
use axum::{Router, routing::get, routing::post};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

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

  let config: Config = resolve_config_from_args()?;
  debug!("config loaded");

  let grpc_state = GrpcState::connect(&config.merchant_grpc_addr).await?;

  let app_state = http::AppState { grpc: grpc_state };

  let blockchain_router = Router::new()
    .route("/chains", get(handlers::list_chains))
    .route("/{id}/assets", get(handlers::list_assets));

  let payment_router = Router::new().route("/intent", post(handlers::create_payment_intent));

  let router: Router = Router::new()
    .nest("/blockchain", blockchain_router)
    .nest("/payment", payment_router)
    .layer(TraceLayer::new_for_http())
    .layer(axum::middleware::from_fn(auth::auth_middleware))
    .with_state(app_state);

  let addr: SocketAddr = config.listen_addr.parse()?;
  info!(%addr, grpc_addr = %config.merchant_grpc_addr, "starting merchant HTTP API");

  let listener = tokio::net::TcpListener::bind(addr).await.map_err(AppError::HttpBind)?;
  let local_addr = listener.local_addr()?;
  info!(%local_addr, grpc_addr = %config.merchant_grpc_addr, "merchant HTTP API started");

  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
  info!("merchant HTTP API stopped");
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
