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
