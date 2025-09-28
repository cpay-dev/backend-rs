use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub listen_addr: String,
  pub root_grpc_addr: String,
  pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  pub host: String,
  pub username: String,
  pub password: String,
  pub database: String,
  pub ssl_mode: String,
  pub tls_cert_path: String,
}

impl DatabaseConfig {
  pub fn conn_string(&self) -> String {
    format!(
      "postgresql://{}:{}@{}/{}?sslmode={}",
      self.username, self.password, self.host, self.database, self.ssl_mode
    )
  }
}
