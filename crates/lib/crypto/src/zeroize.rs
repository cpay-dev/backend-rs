use chacha20poly1305::Key;
use core::ops::{Deref, DerefMut};
use zeroize::Zeroize;

/// A transparent wrapper that zeroizes keys on drop.
#[repr(transparent)]
pub struct ZeroizingKey(Key);

impl ZeroizingKey {
  pub fn new(key: Key) -> Self {
    Self(key)
  }

  pub fn into_inner(self) -> Key {
    self.0
  }
}

impl Drop for ZeroizingKey {
  fn drop(&mut self) {
    self.0.zeroize();
  }
}

impl Deref for ZeroizingKey {
  type Target = Key;

  fn deref(&self) -> &Key {
    &self.0
  }
}

impl DerefMut for ZeroizingKey {
  fn deref_mut(&mut self) -> &mut Key {
    &mut self.0
  }
}

impl AsRef<Key> for ZeroizingKey {
  fn as_ref(&self) -> &Key {
    &self.0
  }
}

impl AsMut<Key> for ZeroizingKey {
  fn as_mut(&mut self) -> &mut Key {
    &mut self.0
  }
}

impl From<ZeroizingKey> for Key {
  fn from(t: ZeroizingKey) -> Key {
    t.0
  }
}
