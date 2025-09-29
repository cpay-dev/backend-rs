use crate::config::IndexerConfig;
use crate::error::AppError;
use crate::rpc::Service;
use crate::signals::shutdown_signal;
use crate::types::ConfirmationLevel;
use alloy::consensus::BlockHeader;
use alloy::network::AnyNetwork;
use alloy::network::BlockResponse;
use alloy::network::primitives::BlockTransactions;
use alloy::providers::Provider;
use cpay_proto::cpay::blockchain::v1::indexer;
use tokio::select;
use tracing::{debug, info, trace};

enum LoopStep {
  Continue,
  AwaitHead,
  Publish(indexer::Block),
  Stop,
}

struct IndexState {
  cursor: u64,
  head: u64,
}

pub struct Indexer<P: Provider<AnyNetwork>> {
  rpc: Service<P>,
  js: async_nats::jetstream::Context,
  js_subject: async_nats::Subject,

  confirmation_level: ConfirmationLevel,
  start_at_block: Option<u64>,
  stop_at_block: Option<u64>,

  chain: cpay_proto::cpay::blockchain::v1::Chain,
}

impl<P: Provider<AnyNetwork>> Indexer<P> {
  pub fn new(
    config: &IndexerConfig,
    rpc: Service<P>,
    js: async_nats::jetstream::Context,
    js_subject: async_nats::Subject,
    chain: cpay_proto::cpay::blockchain::v1::Chain,
  ) -> Self {
    Self {
      rpc,
      js,
      js_subject,
      confirmation_level: config.confirmation_level,
      start_at_block: config.start_at_block,
      stop_at_block: config.stop_at_block,
      chain,
    }
  }

  pub async fn run(&self) -> Result<(), AppError> {
    let head_block = self
      .rpc
      .fetch_block_number(self.confirmation_level)
      .await?
      .ok_or_else(|| AppError::NodeNoHeadBlock)?;

    let start_block = match self.start_at_block {
      Some(n) => n,
      None => head_block,
    };

    info!(
      level = ?self.confirmation_level,
      start = start_block,
      head = head_block,
      "starting indexer"
    );

    let mut state = IndexState {
      cursor: start_block,
      head: head_block,
    };

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
      select! {
        _ = &mut shutdown => {
          info!("shutdown signal received; exiting index loop");
          break;
        }
        step = self.step(&mut state) => {
          let mut sleep = std::time::Duration::from_millis(500);
          match step? {
            LoopStep::Continue => {
              state.cursor += 1;
            }
            LoopStep::AwaitHead => {
              sleep = std::time::Duration::from_millis(750);
            }
            LoopStep::Publish(block) => {
              self.publish_block(block).await?;
              state.cursor += 1;
            }
            LoopStep::Stop => break,
          }
          tokio::time::sleep(sleep).await;
        }
      }
    }

    info!("exiting...");

    Ok(())
  }

  async fn step(&self, state: &mut IndexState) -> Result<LoopStep, AppError> {
    if let Some(stop_at) = self.stop_at_block {
      if state.cursor > stop_at {
        return Ok(LoopStep::Stop);
      }
    }

    if state.cursor > state.head {
      if let Some(new_head) = self.rpc.fetch_block_number(self.confirmation_level).await? {
        trace!(head = new_head, "refreshed head");
        state.head = new_head;
      }
      if state.cursor > state.head {
        debug!(current = state.cursor, head = state.head, "cursor is ahead");
        return Ok(LoopStep::AwaitHead);
      }
    }

    let (current_block, receipts) = self.rpc.fetch_block_with_receipts(state.cursor).await?;

    let current_block = match current_block {
      Some(current) => current,
      None => Err(AppError::NoBlock(state.cursor))?,
    };

    assert_eq!(current_block.header().number(), state.cursor);

    let receipts = match receipts {
      Some(receipts) => receipts,
      None => Err(AppError::NoBlockReceipts(state.cursor))?,
    };

    let transactions = match current_block.transactions() {
      BlockTransactions::Full(txs) => txs,
      _ => Err(AppError::NoFullTransactions)?,
    };

    trace!(block = state.cursor, transactions = transactions.len(), "fetched block");

    assert_eq!(receipts.len(), transactions.len());

    if transactions.is_empty() {
      Ok(LoopStep::Continue)
    } else {
      let block = indexer::Block {
        base: indexer::BlockBase {
          chain: self.chain.into(),
          confirmation_level: cpay_proto::cpay::blockchain::v1::ConfirmationLevel::Pending.into(),
          block_hash: current_block.header().hash.to_string(),
          block_number: state.cursor,
          block_timestamp: current_block.header().timestamp(),
        }.into(),
        transactions: indexer::block::Transactions::EvmTransactions(indexer::EvmTransactions {
          serialized_txs: serde_json::to_vec(&transactions)?,
          serialized_receipts: serde_json::to_vec(&receipts)?,
        })
        .into(),
      };
      Ok(LoopStep::Publish(block))
    }
  }

  async fn publish_block(&self, block: indexer::Block) -> Result<(), AppError> {
    trace!(block = block.base.as_ref().unwrap().block_number, "publishing block");

    use prost::Message;
    let block_data = block.encode_to_vec();
    trace!(
      block = block.base.as_ref().unwrap().block_number,
      data_len = block_data.len(),
      "encoded message"
    );

    _ = self
      .js
      .publish(self.js_subject.clone(), block_data.into())
      .await?
      .await?;
    info!(block = block.base.as_ref().unwrap().block_number, "published message");

    Ok(())
  }
}
