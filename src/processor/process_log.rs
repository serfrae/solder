use super::Processable;
use crate::{
	error::Result,
	models::{ProcessedTransactionLogs, RawTransactionLogs},
};

impl Processable for RawTransactionLogs {
	type Output = ProcessedTransactionLogs;
	fn process(&self) -> Result<Self::Output> {
		unimplemented!();
	}
}
