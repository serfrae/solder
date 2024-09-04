use super::Gettable;
use crate::config::ClientConfig;
use crate::error::{AppError, Result};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig, rpc_response::SlotInfo,
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::{TransactionDetails, UiConfirmedBlock};
use std::future::Future;
use std::pin::Pin;
use tokio::time::{timeout, sleep, Duration};

/// Retrieve block from a slot number and outputs the a tuple of `SlotInfo` and `UiConfirmedBlock`
impl Gettable for SlotInfo {
    type Output = (Self, UiConfirmedBlock);
    fn get(
        input: SlotInfo,
        config: &ClientConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + Send + 'static>> {
        let url = config.get_url();
        Box::pin(async move {
            let client = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());

            // Block for current slot is typically not yet processed, the slot before it (parent)
            // can or cannot be processed testing showed that 2 slots behind is generally the
            // most recent processed block for RPC call
            let slot = input.slot - 2;

            let block_config = RpcBlockConfig {
                encoding: None,
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
                rewards: Some(true),
                transaction_details: Some(TransactionDetails::Full),
            };

            let max_retries = 3;
            let retry_delay = Duration::from_millis(500);
            let total_timeout = Duration::from_secs(10);

            let retry_future = async {
                let mut retries = 0;
                loop {
                    match client
                        .get_block_with_config(slot, block_config.clone())
                        .await
                    {
                        Ok(block) => return Ok((input, block)),
                        Err(e) if retries < max_retries => {
                            retries += 1;
                            log::warn!(
                                "Error getting block (attempt {}): {}. Retrying...",
                                retries, e
                            );
                            sleep(retry_delay).await;
                        }
                        Err(e) => return Err(AppError::from(e)),
                    }
                }
            };

            match timeout(total_timeout, retry_future).await {
                Ok(result) => result,
                Err(_) => Err(AppError::TimeoutError),
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::load_config;

    #[tokio::test]
    async fn test_get_block() {
        let config = load_config("Config.toml").unwrap();
        let url = config.client.get_url();
        let client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
            url,
            CommitmentConfig::confirmed(),
        );
        let slot = client.get_slot().await.unwrap();

        let slot_info = SlotInfo {
            parent: 123345,
            slot,
            root: 12312,
        };

        let (slot_result, block_result) = SlotInfo::get(slot_info, &config.client).await.unwrap();

        assert!(slot_result.slot > 0);
        assert!(block_result.blockhash != "".to_string());
    }
}
