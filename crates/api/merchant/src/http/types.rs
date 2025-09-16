use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChainDto {
  pub id: String,
  pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetDto {
  pub id: String,
  pub chain: String,
  pub name: String,
  pub symbol: String,
  pub address: String,
  pub decimals: u32,
}
