use crate::error::Result;

pub trait Processable: Send + 'static {
	type Output: Send;
	fn process(&self) -> Result<Self::Output>;
}
