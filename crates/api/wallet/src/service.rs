use crate::error::AppError;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use cpay_proto::cpay::api::v1::blockchain::ChainId;
use cpay_proto::cpay::api::v1::kms::WrapKeyRequest;
use lib_crypto::ZeroizingKey;
use tonic::{Request, Response, Status};

use cpay_proto::cpay::api::v1::kms::key_management_service_client::KeyManagementServiceClient;

use cpay_proto::cpay::api::v1::wallet::{
  CreateWalletRequest, CreateWalletResponse,
  wallet_service_server::{WalletService, WalletServiceServer},
};

pub struct WalletServiceImpl {
  kms_client: KeyManagementServiceClient<tonic::transport::Channel>,
}

impl WalletServiceImpl {
  pub fn new(kms_client: KeyManagementServiceClient<tonic::transport::Channel>) -> Self {
    Self { kms_client }
  }

  pub fn into_server(self) -> WalletServiceServer<Self> {
    WalletServiceServer::new(self)
  }
}

#[tonic::async_trait]
impl WalletService for WalletServiceImpl {
  async fn create_wallet(
    &self,
    request: Request<CreateWalletRequest>,
  ) -> Result<Response<CreateWalletResponse>, Status> {
    let req = request.into_inner();

    let chain = ChainId::try_from(req.chain).map_err(|_| Status::invalid_argument("Invalid chain"))?;

    let transit_cipher = XChaCha20Poly1305::new_from_slice(&req.transit_key)
      .map_err(|_| Status::invalid_argument("Invalid transit key"))?;

    match chain {
      ChainId::AnyEvm => {
        let pk = alloy::signers::local::PrivateKeySigner::random();
        let raw_pk = zeroize::Zeroizing::new(pk.to_field_bytes());

        let wrap_transit_key = ZeroizingKey::new(
          XChaCha20Poly1305::generate_key()
            .map_err(AppError::AeadGenerateKey)
            .map_err(map_err)?,
        );
        let wrap_cipher = XChaCha20Poly1305::new(&wrap_transit_key);
        let wrap_encrypted_pk = lib_crypto::encrypt_data(&wrap_cipher, raw_pk.to_vec())
          .map_err(AppError::Crypto)
          .map_err(map_err)?;

        let wrap_resp = self
          .kms_client
          .clone()
          .wrap_key(WrapKeyRequest {
            data: wrap_encrypted_pk,
            transit_key: wrap_transit_key.to_vec(),
          })
          .await?
          .into_inner();

        let wrapped_pk = lib_crypto::decrypt_data(&wrap_cipher, wrap_resp.encrypted_data)
          .map_err(AppError::Crypto)
          .map_err(map_err)?;
        let encrypted_data = lib_crypto::encrypt_data(&transit_cipher, wrapped_pk.to_vec())
          .map_err(AppError::Crypto)
          .map_err(map_err)?;

        Ok(Response::new(CreateWalletResponse {
          kek_version: wrap_resp.version,
          encrypted_private_key: encrypted_data.to_vec(),
          public_key: pk.public_key().to_string(),
        }))
      }
      _ => {
        return Err(Status::invalid_argument("Invalid chain"));
      }
    }
  }
}

fn map_err(e: AppError) -> Status {
  Status::internal(e.to_string())
}
