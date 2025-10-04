use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config load error: {0}")]
  ConfigLoad(#[from] app_config::ConfigLoadError),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("aead generate key error: {0}")]
  AeadGenerateKey(chacha20poly1305::aead::rand_core::OsError),

  #[error("aead error: {0}")]
  Aead(#[from] chacha20poly1305::aead::Error),

  #[error("crypto error: {0}")]
  Crypto(#[from] lib_crypto::Error),

  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),

  #[error("address parse error: {0}")]
  AddrParse(#[from] std::net::AddrParseError),
}
