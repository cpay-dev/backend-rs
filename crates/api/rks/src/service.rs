use crate::crypto::WrappedMasterKey;
use crate::error::AppError;
use tonic::{Request, Response, Status};

use cpay_proto::cpay::api::v1::kms::{
  RewrapKeyRequest, RewrapKeyResponse, UnwrapKeyRequest, UnwrapKeyResponse, WrapKeyRequest, WrapKeyResponse,
  key_management_service_server::{KeyManagementService, KeyManagementServiceServer},
};

pub struct RootKeyService {
  wrapper: WrappedMasterKey,
  version: u32,
}

impl RootKeyService {
  pub fn new(wrapper: WrappedMasterKey, version: u32) -> Self {
    Self { wrapper, version }
  }

  pub fn into_server(self) -> KeyManagementServiceServer<Self> {
    KeyManagementServiceServer::new(self)
  }
}

#[tonic::async_trait]
impl KeyManagementService for RootKeyService {
  async fn wrap_key(&self, request: Request<WrapKeyRequest>) -> Result<Response<WrapKeyResponse>, Status> {
    let req = request.into_inner();

    let (cipher, plaintext) = lib_crypto::decrypt_transit(req.transit_key, req.data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let wrapped = self.wrapper.wrap(plaintext).map_err(map_err)?;

    let encrypted_data = lib_crypto::encrypt_data(&cipher, wrapped.to_vec())
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    Ok(Response::new(WrapKeyResponse {
      version: self.version,
      encrypted_data,
    }))
  }

  async fn unwrap_key(&self, request: Request<UnwrapKeyRequest>) -> Result<Response<UnwrapKeyResponse>, Status> {
    let req = request.into_inner();
    if req.version != self.version {
      return Err(Status::failed_precondition("invalid version"));
    }

    let (cipher, wrapped_key) = lib_crypto::decrypt_transit(req.transit_key, req.data)
      .map_err(AppError::Crypto)
      .map_err(map_err)?;

    let unwrapped = self.wrapper.unwrap(wrapped_key.as_ref()).map_err(map_err)?;

    let encrypted_data = lib_crypto::encrypt_data(&cipher, unwrapped.to_vec())
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

fn map_err(e: AppError) -> Status {
  Status::internal(e.to_string())
}

#[cfg(test)]
mod tests {
  use super::RootKeyService;
  use crate::crypto::WrappedMasterKey;
  use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
  use cpay_proto::cpay::api::v1::kms::{
    UnwrapKeyRequest, WrapKeyRequest, key_management_service_server::KeyManagementService,
  };
  use tonic::Request;

  #[tokio::test]
  async fn wrap_and_unwrap_round_trip() {
    let master_key = zeroize::Zeroizing::new([1u8; 32]);
    let wrapped_master_key = WrappedMasterKey::new(master_key).expect("create wrapped key");
    let version = 1;
    let service = RootKeyService::new(wrapped_master_key, version);

    let transit_key = vec![42u8; 32];
    let transit_cipher = XChaCha20Poly1305::new_from_slice(&transit_key).expect("create transit cipher");

    let plaintext = b"super secret data".to_vec();
    let encrypted_plaintext = lib_crypto::encrypt_data(&transit_cipher, plaintext.clone()).expect("encrypt plaintext");

    let wrap_request = WrapKeyRequest {
      transit_key: transit_key.clone(),
      data: encrypted_plaintext,
    };
    let wrap_response = service
      .wrap_key(Request::new(wrap_request))
      .await
      .expect("wrap key")
      .into_inner();

    assert_eq!(wrap_response.version, version);

    let unwrap_request = UnwrapKeyRequest {
      data: wrap_response.encrypted_data,
      transit_key: transit_key.clone(),
      version,
    };

    let unwrap_response = service
      .unwrap_key(Request::new(unwrap_request))
      .await
      .expect("unwrap key")
      .into_inner();

    let decrypted_plaintext =
      lib_crypto::decrypt_data(&transit_cipher, unwrap_response.decrypted_data).expect("decrypt data");

    assert_eq!(&*decrypted_plaintext, &plaintext);
  }
}
