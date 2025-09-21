use crate::config::ArgonConfig;
use crate::error::AppError;
use argon2::Argon2;

pub fn derive_master_key(
  config: &ArgonConfig,
  secret: &zeroize::Zeroizing<Vec<u8>>,
  password: &zeroize::Zeroizing<Vec<u8>>,
  salt: &[u8; 16],
) -> Result<zeroize::Zeroizing<[u8; 32]>, AppError> {
  let params = argon2::Params::new(config.memory, config.time, config.parallelism, Some(32))?;
  let argon = Argon2::new_with_secret(&secret, argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)?;
  let mut out = zeroize::Zeroizing::new([0u8; 32]);
  argon.hash_password_into(&password, salt, &mut out.as_mut())?;
  Ok(out)
}
