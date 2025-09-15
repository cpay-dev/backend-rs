use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config error: {0}")]
  Config(String),
  #[error("http bind error: {0}")]
  HttpBind(#[from] std::io::Error),
  #[error("invalid address: {0}")]
  Addr(#[from] std::net::AddrParseError),
  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),
  #[error("grpc status: {0}")]
  GrpcStatus(#[from] tonic::Status),
  #[error("serde error: {0}")]
  Serde(#[from] serde_json::Error),
}
