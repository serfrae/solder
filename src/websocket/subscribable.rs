use crate::{config::ClientConfig, error::Result};
use tokio::sync::mpsc;

pub trait Subscribable: Sized {
	type Output;
	fn subscribe(config: &ClientConfig) -> Result<(Self, mpsc::Receiver<Self::Output>)>;
}
