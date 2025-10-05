use crate::error::AppError;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use cpay_proto::cpay::api::v1::kms::WrapKeyRequest;
use cpay_proto::cpay::blockchain::v1::Chain;
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
    let chain = Chain::try_from(req.chain).map_err(|_| Status::invalid_argument("Invalid chain"))?;
    match chain {
      Chain::AnyEvm => {
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

        Ok(Response::new(CreateWalletResponse {
          kek_version: wrap_resp.version,
          encrypted_private_key: wrapped_pk.to_vec(),
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

#[cfg(test)]
mod tests {
  use super::*;
  use alloy::signers::local::PrivateKeySigner;
  use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
  use cpay_proto::cpay::api::v1::kms::{UnwrapKeyRequest, key_management_service_client::KeyManagementServiceClient};
  use cpay_proto::cpay::api::v1::wallet::CreateWalletRequest;
  use lib_crypto::ZeroizingKey;
  use std::{env, error::Error, fs, io};
  use tonic::{
    Request,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint},
  };

  type TestError = Box<dyn Error + Send + Sync + 'static>;
  type TestResult<T> = Result<T, TestError>;

  const KMS_ADDR_ENV: &str = "KMS_GRPC_ADDR";
  const KMS_CA_ENV: &str = "KMS_TLS_CA_CERT";

  struct TestContext {
    service: WalletServiceImpl,
    kms_client: KeyManagementServiceClient<Channel>,
  }

  #[tokio::test]
  async fn create_wallet_with_real_kms() -> TestResult<()> {
    let ctx = create_context().await?;

    let response = ctx
      .service
      .create_wallet(Request::new(CreateWalletRequest {
        chain: Chain::AnyEvm as i32,
        ..Default::default()
      }))
      .await?
      .into_inner();

    assert!(response.kek_version > 0, "kek_version should be positive");
    assert!(!response.encrypted_private_key.is_empty());
    assert!(!response.public_key.is_empty());

    Ok(())
  }

  #[tokio::test]
  async fn decrypts_created_wallet_private_key() -> TestResult<()> {
    let ctx = create_context().await?;

    let create_response = ctx
      .service
      .create_wallet(Request::new(CreateWalletRequest {
        chain: Chain::AnyEvm as i32,
        ..Default::default()
      }))
      .await?
      .into_inner();

    let transit_key =
      ZeroizingKey::new(XChaCha20Poly1305::generate_key().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?);
    let transit_cipher = XChaCha20Poly1305::new(&transit_key);

    let encrypted_payload = lib_crypto::encrypt_data(&transit_cipher, create_response.encrypted_private_key.clone())?;

    let mut kms_client = ctx.kms_client.clone();
    let unwrap_response = kms_client
      .unwrap_key(UnwrapKeyRequest {
        version: create_response.kek_version,
        data: encrypted_payload,
        transit_key: transit_key.to_vec(),
      })
      .await?
      .into_inner();

    let decrypted_key = lib_crypto::decrypt_data(&transit_cipher, unwrap_response.decrypted_data)?;
    let key_bytes: [u8; 32] = decrypted_key
      .as_slice()
      .try_into()
      .map_err(|err: std::array::TryFromSliceError| -> TestError { Box::new(err) })?;

    let signer = PrivateKeySigner::from_slice(&key_bytes)?;
    let derived_public_key = signer.public_key().to_string();

    assert_eq!(derived_public_key, create_response.public_key);

    Ok(())
  }

  async fn create_context() -> TestResult<TestContext> {
    let service_client = connect_kms().await?;
    let kms_client = connect_kms().await?;

    Ok(TestContext {
      service: WalletServiceImpl::new(service_client),
      kms_client,
    })
  }

  async fn connect_kms() -> TestResult<KeyManagementServiceClient<Channel>> {
    let raw_addr = env::var(KMS_ADDR_ENV).map_err(|_| {
      io::Error::new(
        io::ErrorKind::NotFound,
        format!("environment variable {KMS_ADDR_ENV} is required for integration tests"),
      )
    })?;

    let endpoint_builder = Endpoint::from_shared(raw_addr.clone())?;
    let domain = endpoint_builder
      .uri()
      .host()
      .ok_or_else(|| {
        io::Error::new(
          io::ErrorKind::InvalidInput,
          format!("kms address '{raw_addr}' is missing a host"),
        )
      })?
      .to_string();

    let mut tls_config = ClientTlsConfig::new().domain_name(domain);
    if let Ok(ca_path) = env::var(KMS_CA_ENV) {
      let pem = fs::read(ca_path)?;
      let ca = Certificate::from_pem(pem);
      tls_config = tls_config.ca_certificate(ca);
    }

    let channel = endpoint_builder.tls_config(tls_config)?.connect().await?;
    Ok(KeyManagementServiceClient::new(channel))
  }
}
