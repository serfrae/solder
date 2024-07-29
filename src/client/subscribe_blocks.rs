use super::Subscribable;
use crate::{config::ClientConfig, error::Result, models::RawBlock};
use crossbeam_channel::Receiver;
use log::info;
use solana_client::{
	pubsub_client::{BlockSubscription, PubsubClient, PubsubClientSubscription},
	rpc_config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};

impl Subscribable for BlockSubscription {
	type Output = RawBlock;

	fn subscribe(
		config: &ClientConfig,
	) -> Result<(
		PubsubClientSubscription<Self::Output>,
		Receiver<Self::Output>,
	)> {
		let url = if !config.api_key.is_empty() {
			format!("{}?api-key={}", config.url, config.api_key)
		} else {
			config.url.clone()
		};
		info!("Url: {}", url);

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
		info!("Subscribed to blocks");

		Ok((_subscription, rx))
	}
}
