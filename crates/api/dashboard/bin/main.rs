use anyhow::{Context, Result};

use crate::config::Config;
use api_dashboard::{AppState, build_router};
use cpay_proto::cpay::api::v1::authn::authn_service_client::AuthnServiceClient;
use std::time::Duration;
use tonic::transport::Endpoint;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.compact()
		.init();

	let config: Config = app_config::resolve_config_from_args().context("failed to load config")?;

	let endpoint = Endpoint::new(config.authn_service_url)
		.context("invalid authn service url")?
		.connect_timeout(Duration::from_secs(5))
		.timeout(Duration::from_secs(10));
	let authn_client = AuthnServiceClient::connect(endpoint)
		.await
		.context("failed to connect to authn service")?;
	let state = AppState { authn_client };
	let app = build_router(state);

	let listener = tokio::net::TcpListener::bind(&config.listen_addr)
		.await
		.context("failed to bind listen_addr")?;

	tracing::info!(addr = %config.listen_addr, "http server starting");
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await
		.context("server error")?;
	tracing::info!("http server shutting down");

	Ok(())
}

async fn shutdown_signal() {
	#[cfg(unix)]
	{
		let mut term_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("failed to install SIGTERM handler");
		tokio::select! {
			_ = tokio::signal::ctrl_c() => {  }
			_ = term_signal.recv() => {  }
		}
		return;
	}

	#[cfg(not(unix))]
	{
		let _ = tokio::signal::ctrl_c().await;
	}
}
