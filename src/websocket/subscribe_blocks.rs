use super::Subscribable;
use crate::{config::ClientConfig, error::Result, models::RawBlock};
use log::info;
use solana_client::{
	pubsub_client::PubsubClient,
	rpc_config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};
use tokio::sync::mpsc;

pub struct BlockSubscription;

impl Subscribable for BlockSubscription {
	type Output = RawBlock;

	fn subscribe(config: &ClientConfig) -> Result<(Self, mpsc::Receiver<Self::Output>)> {
		let url = if !config.api.key.is_empty() {
			format!("{}?api_key={}", config.url, config.api_key)
		} else {
			config.url.clone()
		};

		let block_filter = RpcBlockSubscribeFilter::All;
		let block_config = RpcBlockSubscribeConfig {
			commitment: Some(CommitmentConfig::confirmed()),
			encoding: Some(UiTransactionEncoding::Json),
			transaction_details: Some(TransactionDetails::Full),
			show_rewards: Some(false),
			max_supported_transaction_version: None,
		};

		info!("Subscribing to blocks...");
		let (_subscription, rx) =
			PubsubClient::block_subscribe(url.as_str(), block_filter, Some(block_config))?;

		Ok((BlockSubscription, rx))
	}
}
