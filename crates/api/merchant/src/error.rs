use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config file read error: {0}")]
  ConfigFileRead(std::io::Error),
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("missing -config file path")]
  MissingConfigPath,
  #[error("http bind error: {0}")]
  HttpBind(std::io::Error),
  #[error("invalid address: {0}")]
  Addr(#[from] std::net::AddrParseError),
  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),
  #[error("grpc status: {0}")]
  Grpc(#[from] tonic::Status),
  #[error("serde error: {0}")]
  Serde(#[from] serde_json::Error),
}
