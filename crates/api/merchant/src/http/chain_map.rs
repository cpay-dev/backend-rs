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

impl ChainCode {
  pub const fn as_str(self) -> &'static str {
    match self {
      ChainCode::ChainAny => "CHAIN_ANY",
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
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "CHAIN_ANY" => Ok(ChainCode::ChainAny),
      "ANY_BTC" => Ok(ChainCode::AnyBtc),
      "ANY_EVM" => Ok(ChainCode::AnyEvm),
      "ANY_SVM" => Ok(ChainCode::AnySvm),
      "BTC" => Ok(ChainCode::Btc),
      "ETH" => Ok(ChainCode::Eth),
      "ARB" => Ok(ChainCode::Arb),
      "POLYGON" => Ok(ChainCode::Polygon),
      "UNI" => Ok(ChainCode::Uni),
      "SOL" => Ok(ChainCode::Sol),
      _ => Err(()),
    }
  }
}

pub fn chain_code_from_proto_enum(e: cpay::blockchain::v1::Chain) -> Result<ChainCode, ApiHttpError> {
  use cpay::blockchain::v1::Chain as E;
  match e {
    E::Unspecified => Err(ApiHttpError::Internal("chain unspecified".into())),
    E::Any => Ok(ChainCode::ChainAny),
    E::AnyBtc => Ok(ChainCode::AnyBtc),
    E::AnyEvm => Ok(ChainCode::AnyEvm),
    E::AnySvm => Ok(ChainCode::AnySvm),
    E::BtcBitcoin => Ok(ChainCode::Btc),
    E::EvmEthereum => Ok(ChainCode::Eth),
    E::EvmArbitrum => Ok(ChainCode::Arb),
    E::EvmPolygon => Ok(ChainCode::Polygon),
    E::EvmUnichain => Ok(ChainCode::Uni),
    E::SvmSolana => Ok(ChainCode::Sol),
  }
}

pub fn proto_enum_from_chain_code(code: ChainCode) -> cpay::blockchain::v1::Chain {
  use cpay::blockchain::v1::Chain as E;
  match code {
    ChainCode::ChainAny => E::Any,
    ChainCode::AnyBtc => E::AnyBtc,
    ChainCode::AnyEvm => E::AnyEvm,
    ChainCode::AnySvm => E::AnySvm,
    ChainCode::Btc => E::BtcBitcoin,
    ChainCode::Eth => E::EvmEthereum,
    ChainCode::Arb => E::EvmArbitrum,
    ChainCode::Polygon => E::EvmPolygon,
    ChainCode::Uni => E::EvmUnichain,
    ChainCode::Sol => E::SvmSolana,
  }
}

pub fn chain_code_str_from_proto_value(id: i32) -> Result<String, ApiHttpError> {
  let e = cpay::blockchain::v1::Chain::try_from(id)
    .map_err(|err| ApiHttpError::Internal(format!("unknown chain id {}: {}", id, err)))?;
  let code = chain_code_from_proto_enum(e)?;
  Ok(code.as_str().to_string())
}

pub fn proto_enum_from_code_str(s: &str) -> Result<cpay::blockchain::v1::Chain, ApiHttpError> {
  match s.parse::<ChainCode>() {
    Ok(code) => Ok(proto_enum_from_chain_code(code)),
    Err(_) => Err(ApiHttpError::BadRequest("invalid chain id".into())),
  }
}
