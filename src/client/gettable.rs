use crate::config::ClientConfig;
use crate::error::Result;
use std::future::Future;
use std::pin::Pin;

pub trait Gettable: Send + 'static {
	type Output;
	fn get(input: Self, config: &ClientConfig) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + 'static + Send>>;
}
