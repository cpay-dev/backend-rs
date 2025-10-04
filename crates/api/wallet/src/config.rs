use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub listen_addr: String,
  pub kms_grpc_addr: String,
}
