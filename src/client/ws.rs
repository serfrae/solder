use super::Subscribable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::queue::SegQueue;
use log::{error, info};
use std::sync::Arc;

pub struct WsClient<T: Subscribable> {
	pub config: ClientConfig,
	pub queue: Arc<SegQueue<T::Output>>,
}

impl<T: Subscribable> WsClient<T> {
	pub fn new(config: ClientConfig, queue: Arc<SegQueue<T::Output>>) -> Self {
		Self { config, queue }
	}

	pub async fn subscribe(&self) -> Result<()> {
		let (_sub, rx) = T::subscribe(&self.config)?;
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
