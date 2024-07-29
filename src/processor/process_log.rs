use super::Processable;
use crate::{
	error::Result,
	models::{ProcessedTransactionLogs, RawTransactionLogs},
};

impl Processable for RawTransactionLogs {
	type ProcessedOutput = ProcessedTransactionLogs;
	fn process(&self) -> Result<Self::ProcessedOutput> {
		unimplemented!();
	}
}
