use super::Subscribable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::channel::{bounded, Receiver};
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

		let (stop_tx, stop_rx) = bounded::<()>(1);

		let queue_clone = self.queue.clone();
		tokio::task::spawn_blocking(move || {
			Self::receive_loop(rx, queue_clone, stop_rx);
		});

		tokio::signal::ctrl_c().await?;
		info!("Ctrl+C received, shutting down websocket...");
		let _ = stop_tx.send(());

		Ok(())
	}

	fn receive_loop(
		rx: Receiver<T::Output>,
		queue: Arc<SegQueue<T::Output>>,
		stop_rx: Receiver<()>,
	) {
		loop {
			crossbeam::select! {
				recv(rx) -> result => {
					match result {
						Ok(response) => {
							info!("Received data");
							queue.push(response);
						}
						Err(e) => {
							error!("Subscription channel closed: {}", e);
							break;
						}
					}
				}
				recv(stop_rx) -> _ => {
					info!("Stop signal received, exiting received loop");
					break;
				}
			}
		}
	}
}
