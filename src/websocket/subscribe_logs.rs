use super::Subscribable;
use crate::{config::ClientConfig, error::Result, models::RawTransactionLogs};
use log::info;
use solana_client::{
	pubsub_client::PubsubClient,
	rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::sync::mpsc;

pub struct TransactionLogsSubscription;

impl Subscribable for TransactionLogsSubscription {
	type Output = RawTransactionLogs;

	fn subscribe(config: &ClientConfig) -> Result<(Self, mpsc::Receiver<Self::Output>)> {
		let url = if !config.api_key.is_empty() {
			format!("{}?api_key={}", config.url, config.api_key)
		} else {
			config.url.clone()
		};

		let log_filter = RpcTransactionLogsFilter::All;
		let log_config = RpcTransactionLogsConfig {
			commitment: Some(CommitmentConfig::confirmed()),
		};

		info!("Subscribing to logs...");
		let (_subscription, rx) =
			PubsubClient::log_subscribe(url.as_str(), log_filter, Some(log_config))?;

		Ok((TransactionLogsSubscription, rx))
	}
}
