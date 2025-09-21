use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, aead::Aead};

use crate::Error;

pub fn decrypt_transit(key: Vec<u8>, data: Vec<u8>) -> Result<(XChaCha20Poly1305, zeroize::Zeroizing<Vec<u8>>), Error> {
  let key = zeroize::Zeroizing::new(key);
  let data = zeroize::Zeroizing::new(data);
  let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| Error::InvalidKeyLen(key.len()))?;
  let plaintext = decrypt_data(&cipher, data.to_vec())?;
  Ok((cipher, plaintext))
}

pub fn encrypt_data(cipher: &XChaCha20Poly1305, data: Vec<u8>) -> Result<Vec<u8>, Error> {
  let data = zeroize::Zeroizing::new(data);
  let nonce = XChaCha20Poly1305::generate_nonce().map_err(Error::GenerateNonce)?;
  let ciphertext = cipher.encrypt(&nonce, data.as_ref())?;
  let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
  out.extend_from_slice(&nonce);
  out.extend_from_slice(&ciphertext);
  Ok(out)
}

pub fn decrypt_data(cipher: &XChaCha20Poly1305, data: Vec<u8>) -> Result<zeroize::Zeroizing<Vec<u8>>, Error> {
  let data = zeroize::Zeroizing::new(data);
  if data.len() < 24 {
    return Err(Error::CiphertextTooShort(data.len()));
  }
  let (nonce, ciphertext) = data.split_at(24);
  let nonce = chacha20poly1305::XNonce::try_from(nonce).map_err(|_| Error::InvalidNonceLen(nonce.len()))?;
  let plaintext = zeroize::Zeroizing::new(cipher.decrypt(&nonce, ciphertext)?);
  Ok(plaintext)
}
