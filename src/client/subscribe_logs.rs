use super::Subscribable;
use crate::{config::ClientConfig, error::Result};
use crossbeam_channel::Receiver;
use log::info;
use solana_client::{
	pubsub_client::{LogsSubscription, PubsubClient, PubsubClientSubscription},
	rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;

impl Subscribable for LogsSubscription {
	type Output = RawTransactionLogs;

	fn subscribe(
		config: &ClientConfig,
	) -> Result<(
		PubsubClientSubscription<Self::Output>,
		Receiver<Self::Output>,
	)> {
        let url = config.get_ws_url();

		let log_filter = RpcTransactionLogsFilter::Mentions(vec!["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()]);
		let log_config = RpcTransactionLogsConfig {
			commitment: Some(CommitmentConfig::finalized()),
		};

		info!("Subscribing to logs...");
		let (log_subscription, log_rx) =
			PubsubClient::logs_subscribe(url.as_str(), log_filter, log_config)?;

		Ok((log_subscription, log_rx))
	}
}
