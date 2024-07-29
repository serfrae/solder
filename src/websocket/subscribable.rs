use crate::{config::ClientConfig, error::Result};
use crossbeam_channel::Receiver;

pub trait Subscribable: Sized {
	type Output;
	fn subscribe(config: &ClientConfig) -> Result<(Self, Receiver<Self::Output>)>;
}
