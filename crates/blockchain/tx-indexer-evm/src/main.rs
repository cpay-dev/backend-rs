mod config;
mod error;
mod indexer;
mod rpc;
mod signals;
mod types;

use crate::error::AppError;
use crate::indexer::Indexer;
use crate::{config::Config, rpc::Service};
use alloy::providers::ProviderBuilder;
use app_config::resolve_config_from_args;
use async_nats::ConnectOptions;
use tracing::{debug, error};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to start indexer");
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

  let mut providers = Vec::with_capacity(config.indexer.nodes.len());
  for u in &config.indexer.nodes {
    providers.push(
      ProviderBuilder::new()
        .network::<alloy::network::AnyNetwork>()
        .connect(u)
        .await
        .map_err(AppError::ProviderConnect)?,
    );
  }
  debug!("collected providers");

  let nats =
    async_nats::connect_with_options(config.nats.url, ConnectOptions::new().require_tls(true).tls_first()).await?;
  debug!("connected to nats");

  Indexer::new(
    &config.indexer,
    Service::new(providers),
    async_nats::jetstream::new(nats),
    config.nats.subject.into(),
    config.chain.0,
  )
  .run()
  .await
}
