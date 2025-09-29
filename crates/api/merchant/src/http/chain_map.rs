use cpay_proto::cpay;
use serde::{Deserialize, Serialize};

use crate::http::error::ApiHttpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
