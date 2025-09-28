use alloy::eips::BlockNumberOrTag;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfirmationLevel {
  Pending,
  Safe,
  Finalized,
}

impl From<ConfirmationLevel> for BlockNumberOrTag {
  fn from(value: ConfirmationLevel) -> Self {
    match value {
      ConfirmationLevel::Pending => BlockNumberOrTag::Pending,
      ConfirmationLevel::Safe => BlockNumberOrTag::Safe,
      ConfirmationLevel::Finalized => BlockNumberOrTag::Finalized,
    }
  }
}

impl fmt::Display for ConfirmationLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      Self::Finalized => f.pad("finalized"),
      Self::Safe => f.pad("safe"),
      Self::Pending => f.pad("pending"),
    }
  }
}

impl fmt::Debug for ConfirmationLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ChainSerde(pub cpay_proto::cpay::blockchain::v1::Chain);

impl serde::Serialize for ChainSerde {
  fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    s.serialize_str(self.0.as_str_name())
  }
}

impl<'de> serde::Deserialize<'de> for ChainSerde {
  fn deserialize<D>(d: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
      S(String),
      I(i32),
    }

    match Repr::deserialize(d)? {
      Repr::S(s) => cpay_proto::cpay::blockchain::v1::Chain::from_str_name(&s)
        .map(ChainSerde)
        .ok_or_else(|| serde::de::Error::custom(format!("invalid chain: {s}"))),
      Repr::I(i) => cpay_proto::cpay::blockchain::v1::Chain::try_from(i)
        .map(ChainSerde)
        .map_err(|e| serde::de::Error::custom(e)),
    }
  }
}
