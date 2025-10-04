use serde::{Deserialize, Serialize};

use super::types::{ChainCode, PaymentIntentStatus};

#[derive(Debug, Clone, Serialize)]
pub struct ChainDto {
  pub id: ChainCode,
  pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetDto {
  pub id: String,
  pub chain: ChainCode,
  pub name: String,
  pub symbol: String,
  pub address: String,
  pub decimals: u32,
  pub is_stable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentIntentDto {
  pub id: String,
  pub status: PaymentIntentStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePaymentIntentRequest {
  pub asset_id: String,
  #[serde(default)]
  pub amount_usd: Option<AmountValue>,
  #[serde(default)]
  pub amount_asset: Option<AmountValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AmountValue {
  String(String),
  Uint(u64),
}

impl AmountValue {
  pub fn to_string_normalized(&self) -> String {
    match self {
      AmountValue::String(s) => s.clone(),
      AmountValue::Uint(n) => n.to_string(),
    }
  }
}
