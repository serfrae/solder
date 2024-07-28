use super::Subscribable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::queue::SegQueue;
use log::{error, info};
use std::sync::Arc;

pub struct Client<T: Subscribable> {
	pub config: ClientConfig,
	queue: Arc<SegQueue<T>>,
}

impl<T: Subscribable> Client<T> {
	pub fn new(config: ClientConfig, queue: Arc<SegQueue<T>>) -> Self {
		Self { config, queue }
	}

	pub async fn subscribe(&self) -> Result<()> {
		let (_, mut rx) = T::subscribe(&self.config)?;

		info!("Listening for updates...");
		loop {
			match rx.recv().await {
				Some(response) => {
					info!("Received data");
					&self.queue.push(response);
				}
				None => {
					error!("Subscription channel closed");
					break;
				}
			}
		}
	}
}
