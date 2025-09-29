use serde::{Deserialize, Serialize};

use crate::http::chain_map::ChainCode;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentIntentStatus {
  AwaitingPayment,
  Paid,
  Expired,
  AmlCheckPending,
  AmlCheckFailed,
  RefundPending,
  Refunded,
}

impl PaymentIntentStatus {
  pub const fn as_str(self) -> &'static str {
    match self {
      PaymentIntentStatus::AwaitingPayment => "AWAITING_PAYMENT",
      PaymentIntentStatus::Paid => "PAID",
      PaymentIntentStatus::Expired => "EXPIRED",
      PaymentIntentStatus::AmlCheckPending => "AML_CHECK_PENDING",
      PaymentIntentStatus::AmlCheckFailed => "AML_CHECK_FAILED",
      PaymentIntentStatus::RefundPending => "REFUND_PENDING",
      PaymentIntentStatus::Refunded => "REFUNDED",
    }
  }
}

impl core::fmt::Display for PaymentIntentStatus {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
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
