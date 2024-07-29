use super::Subscribable;
use crate::{config::ClientConfig, error::Result, models::RawTransactionLogs};
use crossbeam_channel::Receiver;
use log::info;
use solana_client::{
	pubsub_client::PubsubClient,
	rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;

pub struct TransactionLogsSubscription;

impl Subscribable for TransactionLogsSubscription {
	type Output = RawTransactionLogs;

	fn subscribe(config: &ClientConfig) -> Result<(Self, Receiver<Self::Output>)> {
		let url = if !config.api_key.is_empty() {
			format!("{}?api-key={}", config.url, config.api_key)
		} else {
			config.url.clone()
		};

		let log_filter = RpcTransactionLogsFilter::All;
		let log_config = RpcTransactionLogsConfig {
			commitment: Some(CommitmentConfig::confirmed()),
		};

		info!("Subscribing to logs...");
		let log_subscribe = PubsubClient::logs_subscribe(url.as_str(), log_filter, log_config);
		let (_subscription, rx) = log_subscribe?;

		Ok((TransactionLogsSubscription, rx))
	}
}
