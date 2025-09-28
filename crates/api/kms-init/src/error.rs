use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("config load error: {0}")]
  ConfigLoad(#[from] app_config::ConfigLoadError),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("pem error: {0}")]
  Pem(#[from] rustls::pki_types::pem::Error),

  #[error("serde json error: {0}")]
  Json(#[from] serde_json::Error),

  #[error("failed to add root certificate to root store: {0}")]
  RootCertAdd(String),

  #[error("postgres error: {0}")]
  Pg(#[from] tokio_postgres::Error),

  #[error("aead generate key error: {0}")]
  AeadGenerateKey(chacha20poly1305::aead::rand_core::OsError),

  #[error("crypto error: {0}")]
  Crypto(#[from] lib_crypto::Error),

  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),

  #[error("grpc status: {0}")]
  Grpc(#[from] tonic::Status),

  #[error("address parse error: {0}")]
  AddrParse(#[from] std::net::AddrParseError),

  #[error("join error: {0}")]
  Join(#[from] tokio::task::JoinError),
}
