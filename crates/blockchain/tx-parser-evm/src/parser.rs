use std::collections::HashMap;

use alloy::{
  consensus::Transaction,
  network::{AnyRpcTransaction, AnyTransactionReceipt, ReceiptResponse, TransactionResponse},
  primitives::TxHash,
  sol_types::SolEvent,
};
use cpay_proto::cpay::blockchain::v1::indexer;
use futures_util::TryStreamExt;
use tokio::select;
use tracing::{info, trace};

use crate::{error::AppError, signals::shutdown_signal, types::Transfer};

pub struct Parser {
  parser: BlockParser,
  js: async_nats::jetstream::Context,
  js_subject: async_nats::Subject,
}

impl Parser {
  pub fn new(parser: BlockParser, js: async_nats::jetstream::Context, js_subject: async_nats::Subject) -> Self {
    Self { parser, js, js_subject }
  }

  pub async fn parse(&self, stream: &mut async_nats::jetstream::consumer::pull::Stream) -> Result<(), AppError> {
    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
      select! {
          _ = &mut shutdown => {
              info!("shutdown signal received; exiting parser loop");
              break;
          }
          msg = stream.try_next() => {
              if let Some(msg) = msg? {
                self.process_message(msg).await?;
              }
          }
      }
    }

    info!("exiting...");

    Ok(())
  }

  async fn process_message(&self, msg: async_nats::jetstream::Message) -> Result<(), AppError> {
    trace!("message received");
    use prost::Message;
    let block = indexer::Block::decode(msg.payload.as_ref())?;
    let block_number = block.base.as_ref().unwrap().block_number;
    trace!(block = block_number, "block decoded");
    let parsed_block = self.parser.parse(block).await?;
    trace!(block = block_number, transfers = ?parsed_block.transfers, "block parsed");
    self.publish_block(parsed_block).await?;
    info!(block = block_number, "block published");
    msg.ack().await.map_err(|e| AppError::NatsAck(e.to_string()))?;
    info!(block = block_number, "message acknowledged");
    Ok(())
  }

  async fn publish_block(&self, parsed_block: indexer::ParsedBlock) -> Result<(), AppError> {
    trace!(block = parsed_block.block.as_ref().unwrap().block_number, "publishing block");

    use prost::Message;
    let block_data = parsed_block.encode_to_vec();
    trace!(
      block = parsed_block.block.as_ref().unwrap().block_number,
      data_len = block_data.len(),
      "encoded message"
    );

    _ = self
      .js
      .publish(self.js_subject.clone(), block_data.into())
      .await?
      .await?;

    Ok(())
  }
}

pub struct BlockParser {}

impl BlockParser {
  pub fn new() -> Self {
    Self {}
  }

  async fn parse(&self, block: indexer::Block) -> Result<indexer::ParsedBlock, AppError> {
    let block_base = block.base.unwrap();
    trace!(block = block_base.block_number, "parsing block");

    let transactions = match block.transactions.ok_or(AppError::BlockTransactionsEmpty)? {
      indexer::block::Transactions::EvmTransactions(evm) => evm,
      // _ => return Err(AppError::BlockTransactionsNotSerialized),
    };

    let receipts: Vec<AnyTransactionReceipt> = serde_json::from_slice(transactions.serialized_receipts.as_slice())?;
    let transactions: Vec<AnyRpcTransaction> = serde_json::from_slice(transactions.serialized_txs.as_slice())?;

    let mut valid_transactions: HashMap<TxHash, AnyTransactionReceipt> = HashMap::with_capacity(receipts.len());
    for receipt in receipts {
      if receipt.status() {
        valid_transactions.insert(receipt.transaction_hash(), receipt);
      }
    }

    let mut parsed_transfers: Vec<indexer::ParsedTransfer> = Vec::with_capacity(valid_transactions.len());
    for tx in transactions {
      let receipt = match valid_transactions.remove(&tx.tx_hash()) {
        Some(receipt) => receipt,
        None => continue,
      };

      if let Some(to) = receipt.to {
        if tx.value() > 0 {
          parsed_transfers.push(indexer::ParsedTransfer {
            tx_hash: tx.tx_hash().to_string(),
            amount: tx.value().to_string(),
            from: receipt.from().to_string(),
            to: to.to_string(),
            index: 0,
            kind: indexer::parsed_transfer::Kind::Native(true).into(),
          });
        }
      }

      for (index, log) in receipt.logs().iter().enumerate() {
        if log.topic0() != Some(&crate::types::Transfer::SIGNATURE_HASH) {
          continue;
        }
        let decoded = Transfer::decode_log_validate(&log.inner);
        match decoded {
          Ok(decoded) => {
            parsed_transfers.push(indexer::ParsedTransfer {
              tx_hash: tx.tx_hash().to_string(),
              amount: decoded.value.to_string(),
              from: decoded.from.to_string(),
              to: decoded.to.to_string(),
              index: u64::try_from(index).expect("index too large"),
              kind: indexer::parsed_transfer::Kind::Contract(log.address().to_string()).into(),
            });
          }
          Err(_) => {
            continue;
          }
        }
      }
    }

    Ok(indexer::ParsedBlock {
      block: block_base.into(),
      transfers: parsed_transfers,
    })
  }
}
