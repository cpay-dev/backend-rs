mod config;
mod error;
mod parser;
mod signals;
mod types;

use crate::config::Config;
use crate::error::AppError;
use crate::parser::{BlockParser, Parser};
use app_config::resolve_config_from_args;
use async_nats::ConnectOptions;
use async_nats::jetstream::consumer::pull;
use tracing::{debug, error};

#[tokio::main]
async fn main() {
  if let Err(e) = start().await {
    error!(error = ?e, "failed to start parser");
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

  let client =
    async_nats::connect_with_options(config.nats_url, ConnectOptions::new().require_tls(true).tls_first()).await?;
  debug!("connected to nats");

  let jetstream = async_nats::jetstream::new(client);

  let consumer = jetstream
    .create_consumer_strict_on_stream(
      pull::Config {
        durable_name: config.nats_consumer.durable_name,
        filter_subject: config.nats_consumer.subject,
        ack_wait: std::time::Duration::from_secs(5),
        ..Default::default()
      },
      config.nats_consumer.stream,
    )
    .await?;
  debug!("consumer created");

  let block_parser = BlockParser::new();
  let parser = Parser::new(block_parser, jetstream, config.nats_producer.subject.into());

  parser.parse(&mut consumer.messages().await?).await
}
