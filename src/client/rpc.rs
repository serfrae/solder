use super::Gettable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::queue::SegQueue;
use log::info;
use std::sync::Arc;

pub struct RpcClient<T: Gettable> {
	pub config: ClientConfig,
	pub queue_in: Arc<SegQueue<T>>,
	pub queue_out: Arc<SegQueue<T::Output>>,
}

impl<T: Gettable> RpcClient<T> {
	pub fn new(
		config: ClientConfig,
		queue_in: Arc<SegQueue<T>>,
		queue_out: Arc<SegQueue<T::Output>>,
	) -> Self {
		let url = config.get_url();
		info!("Url: {}", url);

		Self {
			config,
			queue_in,
			queue_out,
		}
	}

	pub async fn run(&mut self) -> Result<()> {
		loop {
			match self.queue_in.pop() {
				Some(data) => {
					let output = T::get(data, &self.config).await?;
					self.queue_out.push(output);
				}
				None => tokio::task::yield_now().await,
			}
		}
	}
}
