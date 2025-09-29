use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config load error: {0}")]
  ConfigLoad(#[from] app_config::ConfigLoadError),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("http bind error: {0}")]
  HttpBind(std::io::Error),

  #[error("invalid address: {0}")]
  Addr(#[from] std::net::AddrParseError),

  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),

  #[error("grpc status: {0}")]
  Grpc(#[from] tonic::Status),
}
