use crate::error::AppError;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub kms_grpc_addr: String,
}

impl Config {
  pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, AppError> {
    let path_ref = path.as_ref();
    let raw = std::fs::read_to_string(path_ref).map_err(AppError::ConfigFileRead)?;
    let cfg: Config = serde_json::from_str(&raw)?;
    Ok(cfg)
  }
}
