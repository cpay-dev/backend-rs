use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config load error: {0}")]
  ConfigLoad(#[from] app_config::ConfigLoadError),

  #[error("provider connect error: {0}")]
  ProviderConnect(#[from] alloy::transports::TransportError),

  #[error("nats connect error: {0}")]
  NatsConnect(#[from] async_nats::ConnectError),

  #[error("nats publish error: {0}")]
  NatsPublishError(#[from] async_nats::error::Error<async_nats::jetstream::context::PublishErrorKind>),

  #[error("no head block")]
  NodeNoHeadBlock,

  #[error("fetch block number error: {0}")]
  FetchBlockNumber(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

  #[error("fetch block error: {0}")]
  FetchBlock(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

  #[error("fetch block receipts error: {0}")]
  FetchBlockReceipts(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

  #[error("send batch error: {0}")]
  SendBatch(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

  #[error("no block: {0}")]
  NoBlock(u64),

  #[error("no block receipts: {0}")]
  NoBlockReceipts(u64),

  #[error("no full transactions")]
  NoFullTransactions,

  #[error("serde json error: {0}")]
  SerdeJson(#[from] serde_json::Error),
}
