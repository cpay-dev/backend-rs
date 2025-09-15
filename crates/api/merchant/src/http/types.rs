use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChainDto {
  pub id: String,
  pub name: String,
}
