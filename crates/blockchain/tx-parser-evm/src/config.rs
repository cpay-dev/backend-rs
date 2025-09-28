use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub nats_url: String,
  pub nats_consumer: NatsConsumerConfig,
  pub nats_producer: NatsProducerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConsumerConfig {
  pub stream: String,
  pub subject: String,
  pub durable_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsProducerConfig {
  pub subject: String,
}
