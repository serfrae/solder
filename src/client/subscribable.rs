use crate::{config::ClientConfig, error::Result};
use crossbeam_channel::Receiver;
use solana_client::pubsub_client::PubsubClientSubscription;
use serde::de::DeserializeOwned;

pub trait Subscribable: Sized + 'static {
	type Output: DeserializeOwned;
	fn subscribe(
		config: &ClientConfig,
	) -> Result<(
		PubsubClientSubscription<Self::Output>,
		Receiver<Self::Output>,
	)>;
}
