use super::Processable;
use crate::{
	error::{AppError, Result},
	models::{ProcessedBlock, ProcessedTransaction},
};
use solana_client::rpc_response::SlotInfo;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiConfirmedBlock};

use log::info;

impl Processable for (SlotInfo, UiConfirmedBlock) {
	type Output = (ProcessedBlock, Vec<Option<ProcessedTransaction>>);
	fn process(&self) -> Result<Self::Output> {
		let processed_block = ProcessedBlock::from(self.clone());

		info!("Processing transactions...");
		let processed_transactions: Vec<Option<ProcessedTransaction>> = self
			.1
			.transactions
			.clone()
			.ok_or(AppError::NoData)?
			.into_iter()
			.map(
				|x: EncodedTransactionWithStatusMeta| -> Option<ProcessedTransaction> {
					let ptx = ProcessedTransaction::try_from(x);
					if let Ok(ptx) = ptx {
						Some(ptx)
					} else {
						None
					}
				},
			)
			.collect();

		info!("Block processed: {}", processed_block.blockhash);
		Ok((processed_block, processed_transactions))
	}
}
