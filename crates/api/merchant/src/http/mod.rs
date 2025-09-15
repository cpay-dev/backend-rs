pub mod auth;
pub mod chain_map;
pub mod error;
pub mod handlers;
pub mod types;

use crate::grpc::GrpcState;

#[derive(Clone)]
pub struct AppState {
  pub grpc: GrpcState,
}
