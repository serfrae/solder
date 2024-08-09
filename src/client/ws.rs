use super::Subscribable;
use crate::config::ClientConfig;
use crate::error::Result;
use crossbeam::channel::{bounded, Receiver};
use log::{error, info};

pub struct WsClient<T: Subscribable> {
	pub config: ClientConfig,
	pub rpc_tx: crossbeam_channel::Sender<T::Output>,
}

impl<T: Subscribable> WsClient<T> {
	pub fn new(config: ClientConfig, rpc_tx: crossbeam_channel::Sender<T::Output>) -> Self {
		Self { config, rpc_tx }
	}

	pub async fn subscribe(&self) -> Result<()> {
		let (_sub, rx) = T::subscribe(&self.config)?;
		info!("Listening for updates...");

		let (stop_tx, stop_rx) = bounded::<()>(1);

		let rpc_tx = self.rpc_tx.clone();
		tokio::task::spawn_blocking(move || {
			Self::receive_loop(rpc_tx, rx, stop_rx);
		});

		tokio::signal::ctrl_c().await?;
		info!("Ctrl+C received, shutting down websocket...");
		let _ = stop_tx.send(());

		Ok(())
	}

	fn receive_loop(
		rpc_tx: crossbeam_channel::Sender<T::Output>,
		rx: Receiver<T::Output>,
		stop_rx: Receiver<()>,
	) {
		loop {
			crossbeam::select! {
				recv(rx) -> result => {
					match result {
						Ok(response) => {
							match rpc_tx.send(response) {
							Ok(_) => continue,
							Err(e) => {
								error!("Error sending data: {}", e);
								continue
							}
						};
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
