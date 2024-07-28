use crate::error::Result;

pub trait Processable: Sized + Send {
	type ProcessedOutput;
	fn process(self) -> Result<Self::ProcessedOutput>;
}
