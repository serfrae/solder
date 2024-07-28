use crate::config::ClientConfig;
use crate::error::Result;
use crate::models::Logs;
use crossbeam::queue::SegQueue;
use log::{info, error};
use solana_client::{
	pubsub_client::PubsubClient,
	rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

pub struct Client {
	pub config: ClientConfig,
	queue: Arc<SegQueue<Logs>>,
}

impl Client {
	pub fn new(config: ClientConfig, queue: Arc<SegQueue<Logs>>) -> Self {
		Self { config, queue }
	}

	pub async fn subscribe_logs(&self) -> Result<()> {
		let url = if !self.config.api_key.is_empty() {
			format!("{}?api_key={}", self.config.url, self.config.api_key)
		} else {
			self.config.url.clone()
		};

		let log_filter = RpcTransactionLogsFilter::All;
		let log_config = RpcTransactionLogsConfig {
			commitment: Some(CommitmentConfig::confirmed()),
		};
		info!("Attempting to subscribe to logs...");
		let (_subscription, rx) =
			PubsubClient::logs_subscribe(url.as_str(), log_filter, log_config)?;
		info!("Connection established, subscribed to logs");

		info!("Starting data retrieval loop");
		loop {
			match rx.recv() {
				Ok(response) => {
					//info!("Client received log: {}", response.context.slot);
					self.queue.push(response);
				}
				Err(e) => {
					error!("Failed to receive log: {}", e);
					break;
				}
			}
		}

		Ok(())
	}
}
