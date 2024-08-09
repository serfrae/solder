use crate::error::Result;

/// Trait to ensure that `ProcessingWorker` is reusuable for any type that can be processed
pub trait Processable: Send + 'static {
	type Output: Send;
	fn process(&self) -> Result<Self::Output>;
}
