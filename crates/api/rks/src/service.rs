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
