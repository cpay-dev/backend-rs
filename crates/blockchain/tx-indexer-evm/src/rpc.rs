use crate::error::AppError;
use crate::types::ConfirmationLevel;
use alloy::eips::BlockId;
use alloy::network::{AnyNetwork, Network};
use alloy::providers::Provider;
use alloy::rpc::client::{BatchRequest, Waiter};
use rand::seq::IndexedRandom;

pub struct Service<P: Provider<AnyNetwork>> {
  providers: Vec<P>,
}

impl<P: Provider<AnyNetwork>> Service<P> {
  pub fn new(providers: Vec<P>) -> Self {
    Self { providers }
  }

  pub async fn fetch_block_number(&self, level: ConfirmationLevel) -> Result<Option<u64>, AppError> {
    let provider = self.choose_random_provider();
    let n = provider
      .get_block_number_by_id(BlockId::Number(level.into()))
      .await
      .map_err(AppError::FetchBlockNumber)?;
    Ok(n)
  }

  pub async fn fetch_block_with_receipts(
    &self,
    block: u64,
  ) -> Result<
    (
      Option<<AnyNetwork as Network>::BlockResponse>,
      Option<Vec<<AnyNetwork as Network>::ReceiptResponse>>,
    ),
    AppError,
  > {
    let provider = self.choose_random_provider();

    let mut batch = BatchRequest::new(provider.client());
    let block_id = BlockId::number(block);

    let block_waiter: Waiter<Option<<AnyNetwork as Network>::BlockResponse>> = batch
      .add_call("eth_getBlockByNumber", &(block_id, true))
      .map_err(AppError::FetchBlock)?;

    let receipts_waiter: Waiter<Option<Vec<<AnyNetwork as Network>::ReceiptResponse>>> = batch
      .add_call("eth_getBlockReceipts", &(block_id,))
      .map_err(AppError::FetchBlockReceipts)?;

    batch.send().await.map_err(AppError::SendBatch)?;

    let block = block_waiter.await.map_err(AppError::FetchBlock)?;
    let receipts = receipts_waiter.await.map_err(AppError::FetchBlockReceipts)?;

    Ok((block, receipts))
  }

  #[inline]
  fn choose_random_provider(&self) -> &P {
    self.providers.choose(&mut rand::rng()).expect("no providers available")
  }
}
