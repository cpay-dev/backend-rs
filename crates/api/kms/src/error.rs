use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("pem error: {0}")]
  Pem(#[from] rustls::pki_types::pem::Error),
  #[error("serde json error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("config file read error: {0}")]
  ConfigFileRead(std::io::Error),
  #[error("missing -config file path")]
  MissingConfigPath,
  #[error("no KEKs found in database")]
  NoKeks,
  #[error("no ACTIVE KEK found")]
  NoActiveKek,
  #[error("failed to add root certificate to root store: {0}")]
  RootCertAdd(String),
  #[error("postgres error: {0}")]
  Pg(#[from] tokio_postgres::Error),
  #[error("aead generate key error: {0}")]
  AeadGenerateKey(chacha20poly1305::aead::rand_core::OsError),
  #[error("aead error: {0}")]
  Aead(#[from] chacha20poly1305::aead::Error),
  #[error("crypto error: {0}")]
  Crypto(#[from] lib_crypto::Error),
  #[error("invalid key length: {0}")]
  InvalidKeyLen(usize),
  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),
  #[error("grpc status: {0}")]
  Grpc(#[from] tonic::Status),
  #[error("ulid decode error: {0}")]
  UlidDecode(#[from] ulid::DecodeError),
  #[error("utf8 error: {0}")]
  Utf8(#[from] std::str::Utf8Error),
  #[error("address parse error: {0}")]
  AddrParse(#[from] std::net::AddrParseError),
  #[error("join error: {0}")]
  Join(#[from] tokio::task::JoinError),
  #[error("invalid ciphertext: too short: {0}")]
  CiphertextShort(usize),
}
