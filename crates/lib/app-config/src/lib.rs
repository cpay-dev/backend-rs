use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigLoadError {
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[error("serde json error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("missing -config file path")]
  MissingConfigPath,
}

pub fn resolve_config_path_from_args() -> Result<std::path::PathBuf, ConfigLoadError> {
  let args: Vec<String> = std::env::args().collect();
  let mut path = std::path::PathBuf::from("config.json");
  let mut i = 1;
  while i < args.len() {
    if args[i] == "-config" || args[i] == "--config" {
      if i + 1 >= args.len() {
        return Err(ConfigLoadError::MissingConfigPath);
      }
      path = std::path::PathBuf::from(&args[i + 1]);
      i += 2;
      continue;
    }
    i += 1;
  }
  Ok(path)
}

pub fn resolve_config_from_args<T: DeserializeOwned>() -> Result<T, ConfigLoadError> {
  let path = resolve_config_path_from_args()?;
  let raw = std::fs::read_to_string(path)?;
  let cfg = serde_json::from_str::<T>(&raw)?;
  Ok(cfg)
}
