use super::Subscribable;
use crate::{
    config::ClientConfig,
    error::{AppError, Result},
};
use crossbeam_channel::Receiver;
use rand::Rng;
use solana_client::{
    pubsub_client::{PubsubClient, PubsubClientSubscription, SlotsSubscription},
    rpc_response::SlotInfo,
};
use std::thread::sleep;
use tokio::time::Duration;

/// Retrieves the latest slot which in turn is used to retrieve the latest block
/// Uses the solana pubsub client, which doesn't seem to require ping/pong.
impl Subscribable for SlotsSubscription {
    type Output = SlotInfo;

    fn subscribe(
        config: &ClientConfig,
    ) -> Result<(
        PubsubClientSubscription<Self::Output>,
        Receiver<Self::Output>,
    )> {
        let url = config.get_ws_url();

        let max_retries = 5;
        let initial_backoff = Duration::from_millis(100);
        for attempt in 0..max_retries {
            log::info!(
                "Attempting to subscribe to slots (attempt {}/{})...",
                attempt + 1,
                max_retries
            );

            match PubsubClient::slot_subscribe(url.as_str()) {
                Ok((slot_subscription, slot_rx)) => {
                    log::info!("Successfully subscribed to slots");
                    return Ok((slot_subscription, slot_rx));
                }
                Err(e) if attempt < max_retries - 1 => {
                    log::warn!("Failed to subscribe to slots: {}. Retrying...", e);

                    // Calculate backoff with jitter
                    let backoff = initial_backoff * 2u32.pow(attempt as u32);
                    let jitter = rand::thread_rng().gen_range(0..=100);
                    let delay = backoff + Duration::from_millis(jitter);

                    sleep(delay);
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Err(AppError::Unknown(
            "Failed to subscribe after maximum retries".to_string(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::load_config;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_subscribe() {
        let config = load_config("Config.toml").unwrap();

        let test_timeout = Duration::from_secs(10);

        let result = timeout(test_timeout, async {
            let result = SlotsSubscription::subscribe(&config.client);

            assert!(result.is_ok(), "Subscription failed");
            Ok::<(), Box<dyn std::error::Error>>(())
        }).await;

        assert!(result.is_ok(), "Test failed to complete within the timeout duration");
    }
}

