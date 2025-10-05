use crate::error::AppError;
use crate::repo::KekRecord;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use lib_crypto::ZeroizingKey;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

use cpay_proto::cpay::api::v1::kms::{
  RewrapKeyRequest, RewrapKeyResponse, UnwrapKeyRequest, UnwrapKeyResponse, WrapKeyRequest, WrapKeyResponse,
  key_management_service_client::KeyManagementServiceClient,
  key_management_service_server::{KeyManagementService, KeyManagementServiceServer},
};

pub struct KmsService {
  root_key_client: KeyManagementServiceClient<tonic::transport::Channel>,
  keys: HashMap<u32, KekRecord>,
  active_key: KekRecord,
}

impl KmsService {
  pub fn new(
    root_key_client: KeyManagementServiceClient<tonic::transport::Channel>,
    keys: Vec<KekRecord>,
    active_key: KekRecord,
  ) -> Self {
    let mut map = HashMap::with_capacity(keys.len());
    for k in keys {
      map.insert(k.key_version, k);
    }
    Self {
      root_key_client,
      keys: map,
      active_key,
    }
  }

  pub fn into_server(self) -> KeyManagementServiceServer<Self> {
    KeyManagementServiceServer::new(self)
  }
}

#[tonic::async_trait]
impl KeyManagementService for KmsService {
  async fn wrap_key(&self, request: Request<WrapKeyRequest>) -> Result<Response<WrapKeyResponse>, Status> {
    let req = request.into_inner();
    if req.data.len() < 24 {
      return Err(map_err(AppError::CiphertextShort(req.data.len())));
    }

    let kek_cipher = self.unwrap_kek(&self.active_key).await?;

    let (cipher, plaintext) = lib_crypto::decrypt_transit(req.transit_key, req.data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let wrapped_data = lib_crypto::encrypt_data(&kek_cipher, plaintext.to_vec())
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let encrypted_data = lib_crypto::encrypt_data(&cipher, wrapped_data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    Ok(Response::new(WrapKeyResponse {
      version: self.active_key.key_version,
      encrypted_data,
    }))
  }

  async fn unwrap_key(&self, request: Request<UnwrapKeyRequest>) -> Result<Response<UnwrapKeyResponse>, Status> {
    let req = request.into_inner();
    let key = self.keys.get(&req.version).ok_or(Status::not_found("kek not found"))?;

    let kek_cipher = self.unwrap_kek(key).await?;

    let (cipher, wrapped_data) = lib_crypto::decrypt_transit(req.transit_key, req.data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let unwrapped_data = lib_crypto::decrypt_data(&kek_cipher, wrapped_data.to_vec())
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let encrypted_data = lib_crypto::encrypt_data(&cipher, unwrapped_data.to_vec())
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    Ok(Response::new(UnwrapKeyResponse {
      decrypted_data: encrypted_data,
    }))
  }

  async fn rewrap_key(&self, _request: Request<RewrapKeyRequest>) -> Result<Response<RewrapKeyResponse>, Status> {
    Err(Status::unimplemented("RewrapKey is not implemented"))
  }
}

impl KmsService {
  async fn unwrap_kek(&self, kek: &KekRecord) -> Result<XChaCha20Poly1305, Status> {
    let rks_key = ZeroizingKey::new(
      XChaCha20Poly1305::generate_key()
        .map_err(AppError::AeadGenerateKey)
        .map_err(map_err)?,
    );
    let rks_cipher = XChaCha20Poly1305::new(&rks_key);
    let rks_kek = lib_crypto::encrypt_data(&rks_cipher, kek.encrypted_key.clone())
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let raw_kek_resp = self
      .root_key_client
      .clone()
      .unwrap_key(UnwrapKeyRequest {
        version: kek.root_key_version,
        data: rks_kek.to_vec(),
        transit_key: rks_key.to_vec(),
      })
      .await?
      .into_inner();

    let raw_kek = lib_crypto::decrypt_data(&rks_cipher, raw_kek_resp.decrypted_data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let kek_cipher = XChaCha20Poly1305::new_from_slice(&raw_kek)
      .map_err(|_| AppError::InvalidKeyLen(raw_kek.len()))
      .map_err(map_err)?;

    Ok(kek_cipher)
  }
}

fn map_err(e: AppError) -> Status {
  Status::internal(e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::repo::{KekRecord, KeyStatus};
  use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
  use std::{env, error::Error, fs, io};
  use tonic::{
    Request,
    transport::{Certificate, ClientTlsConfig, Endpoint},
  };

  use cpay_proto::cpay::api::v1::kms::{
    UnwrapKeyRequest, WrapKeyRequest, key_management_service_client::KeyManagementServiceClient,
  };

  type TestError = Box<dyn Error + Send + Sync + 'static>;
  type TestResult<T> = Result<T, TestError>;

  const ROOT_KEY_ADDR_ENV: &str = "RKS_GRPC_ADDR";
  const ROOT_KEY_CA_ENV: &str = "RKS_TLS_CA_CERT";

  struct TestContext {
    service: KmsService,
    kek_record: KekRecord,
    kek_key: Vec<u8>,
  }

  #[tokio::test]
  async fn unwrap_kek_round_trip() -> TestResult<()> {
    let ctx = create_context().await?;

    let kek_cipher = ctx.service.unwrap_kek(&ctx.kek_record).await?;
    let expected_cipher = XChaCha20Poly1305::new_from_slice(&ctx.kek_key)
      .map_err(|_| format!("invalid kek length: {}", ctx.kek_key.len()))?;

    let payload = b"unwrap_kek_round_trip".to_vec();
    let encrypted = lib_crypto::encrypt_data(&kek_cipher, payload.clone())?;
    let decrypted = lib_crypto::decrypt_data(&expected_cipher, encrypted)?;
    let decrypted_bytes: &[u8] = decrypted.as_ref();

    assert_eq!(decrypted_bytes, payload.as_slice());
    Ok(())
  }

  #[tokio::test]
  async fn wrap_key_uses_unwrapped_kek() -> TestResult<()> {
    let ctx = create_context().await?;

    let transit_key = XChaCha20Poly1305::generate_key()?;
    let transit_cipher = XChaCha20Poly1305::new(&transit_key);
    let plaintext = b"wrap_key_uses_unwrapped_kek".to_vec();
    let transit_encrypted = lib_crypto::encrypt_data(&transit_cipher, plaintext.clone())?;

    let response = ctx
      .service
      .wrap_key(Request::new(WrapKeyRequest {
        data: transit_encrypted,
        transit_key: transit_key.to_vec(),
      }))
      .await?
      .into_inner();

    assert_eq!(response.version, ctx.kek_record.key_version);

    let wrapped_data = lib_crypto::decrypt_data(&transit_cipher, response.encrypted_data)?;
    let kek_cipher = XChaCha20Poly1305::new_from_slice(&ctx.kek_key)
      .map_err(|_| format!("invalid kek length: {}", ctx.kek_key.len()))?;
    let unwrapped = lib_crypto::decrypt_data(&kek_cipher, wrapped_data.to_vec())?;
    let unwrapped_bytes: &[u8] = unwrapped.as_ref();

    assert_eq!(unwrapped_bytes, plaintext.as_slice());
    Ok(())
  }

  #[tokio::test]
  async fn unwrap_key_round_trip() -> TestResult<()> {
    let ctx = create_context().await?;

    let kek_cipher = XChaCha20Poly1305::new_from_slice(&ctx.kek_key)
      .map_err(|_| format!("invalid kek length: {}", ctx.kek_key.len()))?;
    let plaintext = b"unwrap_key_round_trip".to_vec();
    let wrapped_data = lib_crypto::encrypt_data(&kek_cipher, plaintext.clone())?;

    let transit_key = XChaCha20Poly1305::generate_key()?;
    let transit_cipher = XChaCha20Poly1305::new(&transit_key);
    let request_data = lib_crypto::encrypt_data(&transit_cipher, wrapped_data)?;

    let response = ctx
      .service
      .unwrap_key(Request::new(UnwrapKeyRequest {
        version: ctx.kek_record.key_version,
        data: request_data,
        transit_key: transit_key.to_vec(),
      }))
      .await?
      .into_inner();

    let decrypted = lib_crypto::decrypt_data(&transit_cipher, response.decrypted_data)?;
    let decrypted_bytes: &[u8] = decrypted.as_ref();
    assert_eq!(decrypted_bytes, plaintext.as_slice());
    Ok(())
  }

  async fn create_context() -> TestResult<TestContext> {
    let raw_addr = env::var(ROOT_KEY_ADDR_ENV).map_err(|_| {
      io::Error::new(
        io::ErrorKind::NotFound,
        format!("environment variable {ROOT_KEY_ADDR_ENV} is required for integration tests"),
      )
    })?;

    let endpoint_builder = Endpoint::from_shared(raw_addr.clone())?;

    let domain = endpoint_builder
      .uri()
      .host()
      .ok_or_else(|| format!("root key address '{raw_addr}' is missing a host"))?
      .to_string();

    let mut tls_config = ClientTlsConfig::new().domain_name(domain);
    if let Ok(ca_path) = env::var(ROOT_KEY_CA_ENV) {
      let pem = fs::read(ca_path)?;
      let ca = Certificate::from_pem(pem);
      tls_config = tls_config.ca_certificate(ca);
    }

    let channel = endpoint_builder.tls_config(tls_config)?.connect().await?;
    let mut root_client = KeyManagementServiceClient::new(channel);

    let kek_key = XChaCha20Poly1305::generate_key()?;
    let kek_key_vec = kek_key.to_vec();
    let transit_key = XChaCha20Poly1305::generate_key()?;
    let transit_cipher = XChaCha20Poly1305::new(&transit_key);

    let transit_payload = lib_crypto::encrypt_data(&transit_cipher, kek_key_vec.clone())?;

    let wrap_response = root_client
      .wrap_key(WrapKeyRequest {
        data: transit_payload,
        transit_key: transit_key.to_vec(),
      })
      .await?
      .into_inner();

    let wrapped = lib_crypto::decrypt_data(&transit_cipher, wrap_response.encrypted_data)?;
    let wrapped_bytes: Vec<u8> = (*wrapped).clone();
    let kek_record = KekRecord {
      key_version: 1,
      root_key_version: wrap_response.version,
      status: KeyStatus::Active,
      encrypted_key: wrapped_bytes,
    };

    let service = KmsService::new(root_client, vec![kek_record.clone()], kek_record.clone());

    Ok(TestContext {
      service,
      kek_record,
      kek_key: kek_key_vec,
    })
  }
}
