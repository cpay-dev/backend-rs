use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub version: u32,
  pub listen_addr: String,
  pub salt: String,
  pub argon: ArgonConfig,
  pub expected_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArgonConfig {
  pub memory: u32,
  pub time: u32,
  pub parallelism: u32,
}
