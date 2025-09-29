use serde::{Deserialize, Serialize};

use crate::http::chain_map::ChainCode;

#[derive(Debug, Clone, Serialize)]
pub struct ChainDto {
  pub id: ChainCode,
  pub name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid PaymentIntentStatus '{input}'")]
pub struct ParsePaymentIntentStatusError {
  input: String,
}

impl ParsePaymentIntentStatusError {
  pub fn new(input: impl Into<String>) -> Self {
    Self { input: input.into() }
  }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl core::str::FromStr for PaymentIntentStatus {
  type Err = ParsePaymentIntentStatusError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let upper = s.trim().to_ascii_uppercase();
    match upper.as_str() {
      "AWAITING_PAYMENT" => Ok(PaymentIntentStatus::AwaitingPayment),
      "PAID" => Ok(PaymentIntentStatus::Paid),
      "EXPIRED" => Ok(PaymentIntentStatus::Expired),
      "AML_CHECK_PENDING" => Ok(PaymentIntentStatus::AmlCheckPending),
      "AML_CHECK_FAILED" => Ok(PaymentIntentStatus::AmlCheckFailed),
      "REFUND_PENDING" => Ok(PaymentIntentStatus::RefundPending),
      "REFUNDED" => Ok(PaymentIntentStatus::Refunded),
      _ => Err(ParsePaymentIntentStatusError::new(s)),
    }
  }
}

impl Serialize for PaymentIntentStatus {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> Deserialize<'de> for PaymentIntentStatus {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    s.parse::<PaymentIntentStatus>()
      .map_err(|e| serde::de::Error::custom(e.to_string()))
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
