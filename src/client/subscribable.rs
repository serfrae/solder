use crate::{config::ClientConfig, error::Result};
use crossbeam_channel::Receiver;
use solana_client::pubsub_client::PubsubClientSubscription;
use serde::de::DeserializeOwned;

/// Trait to ensure that websocket clients can be generic over any type that implements
/// this trait
pub trait Subscribable: Sized + 'static {
	type Output: DeserializeOwned + Send;
	fn subscribe(
		config: &ClientConfig,
	) -> Result<(
		PubsubClientSubscription<Self::Output>,
		Receiver<Self::Output>,
	)>;
}
