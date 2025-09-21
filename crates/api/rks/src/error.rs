use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("serde json error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("config file read error: {0}")]
  ConfigFileRead(std::io::Error),
  #[error("missing -config file path")]
  MissingConfigPath,
  #[error("grpc transport error: {0}")]
  GrpcTransport(#[from] tonic::transport::Error),
  #[error("argon2 error: {0}")]
  Argon2(#[from] argon2::Error),
  #[error("ulid decode error: {0}")]
  UlidDecode(#[from] ulid::DecodeError),
  #[error("aead generate key error: {0}")]
  AeadGenerateKey(chacha20poly1305::aead::rand_core::OsError),
  #[error("aead generate nonce error: {0}")]
  AeadGenerateNonce(chacha20poly1305::aead::rand_core::OsError),
  #[error("crypto error: {0}")]
  Crypto(#[from] lib_crypto::Error),
  #[error("aead error: {0}")]
  Aead(#[from] chacha20poly1305::aead::Error),
  #[error("invalid key length: {0}")]
  InvalidKeyLen(usize),
  #[error("invalid nonce length: {0}")]
  InvalidNonceLen(usize),
  #[error("invalid ciphertext: too short: {0}")]
  CiphertextTooShort(usize),
  #[error("terminal error: {0}")]
  Terminal(String),
  #[error("address parse error: {0}")]
  AddrParse(#[from] std::net::AddrParseError),
}
