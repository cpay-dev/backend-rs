use serde::{Deserialize, Serialize};

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

use crate::http::error::ApiHttpError;
use cpay_proto::cpay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainCode {
  ChainAny,
  AnyBtc,
  AnyEvm,
  AnySvm,
  Btc,
  Eth,
  Arb,
  Polygon,
  Uni,
  Sol,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid ChainCode '{input}'")]
pub struct ParseChainCodeError {
  input: String,
}

impl ParseChainCodeError {
  pub fn new(input: impl Into<String>) -> Self {
    Self { input: input.into() }
  }
}

impl ChainCode {
  pub const fn as_str(self) -> &'static str {
    match self {
      ChainCode::ChainAny => "ANY",
      ChainCode::AnyBtc => "ANY_BTC",
      ChainCode::AnyEvm => "ANY_EVM",
      ChainCode::AnySvm => "ANY_SVM",
      ChainCode::Btc => "BTC",
      ChainCode::Eth => "ETH",
      ChainCode::Arb => "ARB",
      ChainCode::Polygon => "POLYGON",
      ChainCode::Uni => "UNI",
      ChainCode::Sol => "SOL",
    }
  }
}

impl core::fmt::Display for ChainCode {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl core::str::FromStr for ChainCode {
  type Err = ParseChainCodeError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let upper = s.trim().to_ascii_uppercase();
    match upper.as_str() {
      "ANY" => Ok(ChainCode::ChainAny),
      "ANY_BTC" => Ok(ChainCode::AnyBtc),
      "ANY_EVM" => Ok(ChainCode::AnyEvm),
      "ANY_SVM" => Ok(ChainCode::AnySvm),
      "BTC" => Ok(ChainCode::Btc),
      "ETH" => Ok(ChainCode::Eth),
      "ARB" => Ok(ChainCode::Arb),
      "POLYGON" => Ok(ChainCode::Polygon),
      "UNI" => Ok(ChainCode::Uni),
      "SOL" => Ok(ChainCode::Sol),
      _ => Err(ParseChainCodeError::new(s)),
    }
  }
}

impl Serialize for ChainCode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> Deserialize<'de> for ChainCode {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    s.parse::<ChainCode>()
      .map_err(|e| serde::de::Error::custom(e.to_string()))
  }
}

impl From<ChainCode> for cpay::blockchain::v1::Chain {
  fn from(e: ChainCode) -> Self {
    match e {
      ChainCode::ChainAny => cpay::blockchain::v1::Chain::Any,
      ChainCode::AnyBtc => cpay::blockchain::v1::Chain::AnyBtc,
      ChainCode::AnyEvm => cpay::blockchain::v1::Chain::AnyEvm,
      ChainCode::AnySvm => cpay::blockchain::v1::Chain::AnySvm,
      ChainCode::Btc => cpay::blockchain::v1::Chain::BtcBitcoin,
      ChainCode::Eth => cpay::blockchain::v1::Chain::EvmEthereum,
      ChainCode::Arb => cpay::blockchain::v1::Chain::EvmArbitrum,
      ChainCode::Polygon => cpay::blockchain::v1::Chain::EvmPolygon,
      ChainCode::Uni => cpay::blockchain::v1::Chain::EvmUnichain,
      ChainCode::Sol => cpay::blockchain::v1::Chain::SvmSolana,
    }
  }
}

impl TryFrom<cpay::blockchain::v1::Chain> for ChainCode {
  type Error = ApiHttpError;
  fn try_from(e: cpay::blockchain::v1::Chain) -> Result<Self, Self::Error> {
    match e {
      cpay::blockchain::v1::Chain::Any => Ok(ChainCode::ChainAny),
      cpay::blockchain::v1::Chain::AnyBtc => Ok(ChainCode::AnyBtc),
      cpay::blockchain::v1::Chain::AnyEvm => Ok(ChainCode::AnyEvm),
      cpay::blockchain::v1::Chain::AnySvm => Ok(ChainCode::AnySvm),
      cpay::blockchain::v1::Chain::BtcBitcoin => Ok(ChainCode::Btc),
      cpay::blockchain::v1::Chain::EvmEthereum => Ok(ChainCode::Eth),
      cpay::blockchain::v1::Chain::EvmArbitrum => Ok(ChainCode::Arb),
      cpay::blockchain::v1::Chain::EvmPolygon => Ok(ChainCode::Polygon),
      cpay::blockchain::v1::Chain::EvmUnichain => Ok(ChainCode::Uni),
      cpay::blockchain::v1::Chain::SvmSolana => Ok(ChainCode::Sol),
      cpay::blockchain::v1::Chain::Unspecified => Err(ApiHttpError::Internal("chain unspecified".into())),
    }
  }
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
