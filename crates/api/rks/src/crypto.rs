use crate::error::AppError;
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, aead::Aead};
use lib_crypto::ZeroizingKey;

pub struct WrappedMasterKey {
  enc_master_key: zeroize::Zeroizing<Vec<u8>>,
  enc_master_key_nonce: chacha20poly1305::XNonce,
  key_wrap_cipher: XChaCha20Poly1305,
}

impl WrappedMasterKey {
  pub fn new(master_key: zeroize::Zeroizing<[u8; 32]>) -> Result<Self, AppError> {
    let key = ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(AppError::AeadGenerateKey)?);
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XChaCha20Poly1305::generate_nonce().map_err(AppError::AeadGenerateNonce)?;
    let enc = zeroize::Zeroizing::new(cipher.encrypt(&nonce, master_key.as_ref())?);
    Ok(Self {
      enc_master_key: enc,
      enc_master_key_nonce: nonce,
      key_wrap_cipher: cipher,
    })
  }

  #[inline]
  fn decrypt_master_key(&self) -> Result<XChaCha20Poly1305, AppError> {
    let raw = zeroize::Zeroizing::new(
      self
        .key_wrap_cipher
        .decrypt(&self.enc_master_key_nonce, self.enc_master_key.as_ref())?,
    );
    let mk = XChaCha20Poly1305::new_from_slice(&raw).map_err(|_| AppError::InvalidKeyLen(raw.len()))?;
    Ok(mk)
  }

  pub fn wrap(&self, input: zeroize::Zeroizing<Vec<u8>>) -> Result<zeroize::Zeroizing<Vec<u8>>, AppError> {
    let cipher = self.decrypt_master_key()?;
    let nonce = XChaCha20Poly1305::generate_nonce().map_err(AppError::AeadGenerateNonce)?;
    let ciphertext = cipher.encrypt(&nonce, input.as_ref())?;
    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(zeroize::Zeroizing::new(out))
  }

  pub fn unwrap(&self, input: &[u8]) -> Result<zeroize::Zeroizing<Vec<u8>>, AppError> {
    if input.len() < 24 {
      return Err(AppError::CiphertextTooShort(input.len()));
    }
    let (nonce_bytes, ct) = input.split_at(24);
    let nonce =
      chacha20poly1305::XNonce::try_from(nonce_bytes).map_err(|_| AppError::InvalidNonceLen(nonce_bytes.len()))?;
    let cipher = self.decrypt_master_key()?;
    let plaintext = zeroize::Zeroizing::new(cipher.decrypt(&nonce, ct)?);
    Ok(plaintext)
  }
}
