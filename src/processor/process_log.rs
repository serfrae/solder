use super::Processable;
use crate::{
    error::Result,
    models::{RawTransactionLogs, ProcessedTransactionLogs},
};

impl Processable for RawTransactionLogs {
    type ProcessedOutput = ProcessedTransactionLogs;
    fn process(self) -> Result<Self::ProcessedOutput> {
        Ok(ProcessedTransactionLogs {})
    }
}
