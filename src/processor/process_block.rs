use super::Processable;
use crate::{
	error::{AppError, Result},
	models::{Aggregate, FromMsg, TryDecode},
};
use solana_client::rpc_response::SlotInfo;
use solana_transaction_status::{
	EncodedTransactionWithStatusMeta, UiConfirmedBlock, UiTransaction,
};

use log::{error, info};

/// Processes the block into a vector or `Option<Aggregate>` from a UiConfirmedBlock.
/// The transaction field of a `UiConfirmedBlock` is an `Option<EncodedTransactionWithStatusMeta>`
/// thus the need for an `Option<Aggregate>` returns an error if the entire Vec is empty
impl Processable for (SlotInfo, UiConfirmedBlock) {
	type Output = Vec<Option<Aggregate>>;
	fn process(&self) -> Result<Self::Output> {
		let block_time = self.1.block_time.ok_or(AppError::NoData)?;
		let transaction_data: Vec<Option<Aggregate>> = self
			.1
			.transactions
			.clone()
			.ok_or(AppError::NoData)?
			.into_iter()
			.flat_map(|tx: EncodedTransactionWithStatusMeta| {
				match get_accounts_from_transaction(tx) {
					Some((signature, account_keys)) => {
						let aggregates = account_keys
							.into_iter()
							.map(|account| {
								Some(Aggregate {
									blockhash: self.1.blockhash.clone(),
									slot: self.0.slot as i64,
									block_time,
									signature: signature.clone(),
									account,
								})
							})
							.collect::<Vec<_>>();
						Some(aggregates).into_iter().flatten().collect()
					}
					None => Vec::new(),
				}
			})
			.collect();

        // Error if completely empty
		if transaction_data.is_empty() {
			error!("No transaction data");
			return Err(AppError::NoData);
		}

		info!("Block processed: {}", self.1.blockhash);
		Ok(transaction_data)
	}
}

/// Decodes and retrieves transaction signature and associated accounts from an
/// `EncodedTransactionWithStatusMeta` returns `Some<(String, Vec<String>)`, if 
/// `EncodedTransactionWithStatusMeta` is not `None`
fn get_accounts_from_transaction(
	transaction: EncodedTransactionWithStatusMeta,
) -> Option<(String, Vec<String>)> {
	let ui_tx = UiTransaction::try_decode(transaction).ok()?;
	let signature = &ui_tx.signatures[0];
	let account_keys = <Vec<String>>::from_msg(ui_tx.message);
	Some((signature.to_string(), account_keys))
}
