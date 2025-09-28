use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub indexer: IndexerConfig,
  pub nats: NatsConfig,
  pub chain: crate::types::ChainSerde,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexerConfig {
  pub nodes: Vec<String>,
  pub confirmation_level: crate::types::ConfirmationLevel,

  pub start_at_block: Option<u64>,
  pub stop_at_block: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
  pub url: String,
  pub subject: String,
}
