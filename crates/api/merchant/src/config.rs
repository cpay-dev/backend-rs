use crate::error::AppError;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub listen_addr: String,
  pub merchant_grpc_addr: String,
}

impl Config {
  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, AppError> {
    let path_ref = path.as_ref();
    let raw = std::fs::read_to_string(path_ref)
      .map_err(|e| AppError::Config(format!("failed to read {}: {}", path_ref.display(), e)))?;
    let cfg: Config = serde_json::from_str(&raw)?;
    Ok(cfg)
  }
}
