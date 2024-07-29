use super::Subscribable;
use crate::{config::ClientConfig, error::Result};
use crossbeam_channel::Receiver;
use log::info;
use solana_client::{
	pubsub_client::{PubsubClient, PubsubClientSubscription, SlotsSubscription},
	rpc_response::SlotInfo,
};

impl Subscribable for SlotsSubscription {
	type Output = SlotInfo;

	fn subscribe(
		config: &ClientConfig,
	) -> Result<(
		PubsubClientSubscription<Self::Output>,
		Receiver<Self::Output>,
	)> {
		let url = config.get_ws_url();

		info!("Subscribing to slots...");
		let (slot_subscription, slot_rx) = PubsubClient::slot_subscribe(url.as_str())?;

		Ok((slot_subscription, slot_rx))
	}
}
