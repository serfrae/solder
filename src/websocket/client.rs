use super::Subscribable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::queue::SegQueue;
use log::{error, info};
use std::sync::Arc;

pub struct Client<T: Subscribable> {
	pub config: ClientConfig,
	queue: Arc<SegQueue<T::Output>>,
}

impl<T: Subscribable> Client<T> {
	pub fn new(config: ClientConfig, queue: Arc<SegQueue<T::Output>>) -> Self {
		Self { config, queue }
	}

	pub async fn subscribe(&self) -> Result<()> {
		let (_, rx) = T::subscribe(&self.config)?;
        info!("{:?}", rx);

		info!("Listening for updates...");
		loop {
			match rx.recv() {
				Ok(response) => {
					info!("Received data");
					self.queue.push(response);
				}
				Err(e) => {
					error!("Subscription channel closed: {}", e);
					break;
				}
			}
		}
		Ok(())
	}
}
