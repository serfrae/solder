use super::Gettable;
use crate::config::ClientConfig;
use crate::error::{AppError, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig, rpc_response::SlotInfo};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{TransactionDetails, UiConfirmedBlock};
use std::future::Future;
use std::pin::Pin;

impl Gettable for SlotInfo {
	type Output = UiConfirmedBlock;
	fn get(
		input: SlotInfo,
		config: &ClientConfig,
	) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + Send + 'static>> {
		let url = config.get_url();
		Box::pin(async move {
			let client =
				RpcClient::new_with_commitment(url.as_str(), CommitmentConfig::confirmed());
			let slot = input.root;
			let block_config = RpcBlockConfig {
				encoding: None,
				commitment: Some(CommitmentConfig::confirmed()),
				max_supported_transaction_version: Some(0),
				rewards: Some(true),
				transaction_details: Some(TransactionDetails::Full),
			};
			client
				.get_block_with_config(slot, block_config)
				.map_err(|e| AppError::from(e))
		})
	}
}
