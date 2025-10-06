use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub version: u32,
  pub root_grpc_addr: String,
  pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  pub host: String,
  pub domain: Option<String>,
  pub username: String,
  pub password: String,
  pub database: String,
  pub ssl_mode: String,
}

impl DatabaseConfig {
  pub fn conn_string(&self) -> String {
    if let Some(domain) = &self.domain {
      return self.with_domain(domain);
    }

    format!(
      "postgresql://{}:{}@{}/{}?sslmode={}",
      self.username, self.password, self.host, self.database, self.ssl_mode
    )
  }

  pub fn with_domain(&self, domain: impl Into<String>) -> String {
    let (addr, port) = Self::split_host_port(&self.host).expect("host must be a valid domain with port");

    format!(
      "host={} hostaddr={} port={} user={} password={} dbname={} sslmode={}",
      domain.into(), // SNI
      addr,
      port,
      self.username,
      self.password,
      self.database,
      self.ssl_mode
    )
  }

  fn split_host_port(input: &str) -> Option<(String, u16)> {
    use std::net::ToSocketAddrs;
    if let Ok(mut iter) = input.to_socket_addrs() {
      if let Some(addr) = iter.next() {
        return Some((addr.ip().to_string(), addr.port()));
      }
    }

    if let Some((host, port_str)) = input.rsplit_once(':') {
      if let Ok(port) = port_str.parse::<u16>() {
        return Some((host.to_string(), port));
      }
    }

    None
  }
}
