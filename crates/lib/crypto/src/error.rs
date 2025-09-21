#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("ciphertext too short: {0}")]
  CiphertextTooShort(usize),
  #[error("invalid nonce length: {0}")]
  InvalidNonceLen(usize),
  #[error("generate nonce error: {0}")]
  GenerateNonce(chacha20poly1305::aead::rand_core::OsError),
  #[error("invalid key length: {0}")]
  InvalidKeyLen(usize),
  #[error("aead error: {0}")]
  Aead(#[from] chacha20poly1305::aead::Error),
}
