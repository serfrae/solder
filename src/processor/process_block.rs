use super::Processable;
use crate::{
	error::{AppError, Result},
	models::{ProcessedBlock, ProcessedTransaction, TransactionWithSignature},
};
use solana_transaction_status::{
	EncodedConfirmedBlock, EncodedTransactionWithStatusMeta, UiConfirmedBlock,
};

use log::{error, info};

//impl Processable for RawBlock {
//	type ProcessedOutput = (ProcessedBlock, ProcessedTransactions);
//	fn process(&self) -> Result<Self::ProcessedOutput> {
//		let processed_block = ProcessedBlock::try_from(self)?;
//		let block = if let Some(block) = &self.value.block {
//			block
//		} else {
//			error!("No block data");
//			return Err(AppError::NoData);
//		};
//
//		info!("Processing transactions...");
//		let processed_transactions: ProcessedTransactions = block
//			.transactions
//			.clone()
//			.ok_or(AppError::NoData)?
//			.into_iter()
//			.zip(
//				block
//					.signatures
//					.clone()
//					.ok_or(AppError::NoData)?
//					.into_iter(),
//			)
//			.map(|(transaction, signature)| TransactionWithSig {
//				transaction,
//				signature,
//			})
//			.filter_map(|tx_with_sig| ProcessedTransactionWithSig::try_from(tx_with_sig).ok())
//			.collect();
//
//		Ok((processed_block, processed_transactions))
//	}
//}

impl Processable for EncodedConfirmedBlock {
	type Output = (ProcessedBlock, Vec<ProcessedTransaction>);
	fn process(&self) -> Result<Self::Output> {
		let processed_block = ProcessedBlock::try_from(self)?;

		info!("Processing transactions...");
		let processed_transactions: Vec<ProcessedTransaction> = self
			.transactions
			.clone()
			.into_iter()
			.map(
				|x: EncodedTransactionWithStatusMeta| -> ProcessedTransaction {
					ProcessedTransaction::try_from(x).expect("placeholder")
				},
			)
			.collect();

		Ok((processed_block, processed_transactions))
	}
}

impl Processable for UiConfirmedBlock {
	type Output = (ProcessedBlock, Vec<ProcessedTransaction>);
	fn process(&self) -> Result<Self::Output> {
		let processed_block = ProcessedBlock::try_from(self.clone())?;

		info!("Processing transactions...");
		info!("Signatures: {:?}", &self.signatures);

		let processed_transactions: Vec<ProcessedTransaction> = self
			.transactions
			.clone()
			.ok_or(AppError::NoData)?
			.into_iter()
			.map(
				|x: EncodedTransactionWithStatusMeta| -> ProcessedTransaction {
					ProcessedTransaction::try_from(x).expect("placeholder")
				},
			)
			.collect();

		info!("Block processed: {}", processed_block.blockhash);
		Ok((processed_block, processed_transactions))
	}
}
