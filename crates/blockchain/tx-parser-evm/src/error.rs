use async_nats::error::Error as NatsError;
use async_nats::jetstream;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config load error: {0}")]
  ConfigLoad(#[from] app_config::ConfigLoadError),

  #[error("nats connect error: {0}")]
  NatsConnect(#[from] async_nats::ConnectError),

  #[error("nats create consumer strict error: {0}")]
  NatsCreateConsumerStrict(#[from] NatsError<jetstream::stream::ConsumerCreateStrictErrorKind>),

  #[error("nats consumer stream error: {0}")]
  NatsConsumerStream(#[from] NatsError<jetstream::consumer::StreamErrorKind>),

  #[error("nats consumer messages error: {0}")]
  NatsConsumerMessages(#[from] NatsError<jetstream::consumer::pull::MessagesErrorKind>),

  #[error("nats publish error: {0}")]
  NatsPublishError(#[from] async_nats::error::Error<async_nats::jetstream::context::PublishErrorKind>),

  #[error("nats ack error: {0}")]
  NatsAck(String),

  #[error("prost decode error: {0}")]
  ProstDecode(#[from] prost::DecodeError),

  #[error("block transactions empty")]
  BlockTransactionsEmpty,

  // #[error("block transactions not serialized")]
  // BlockTransactionsNotSerialized,

  #[error("serde json error: {0}")]
  SerdeJson(#[from] serde_json::Error),
}
