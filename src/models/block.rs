use crate::error::{AppError, Result};
use log::error;
use solana_transaction_status::{EncodedConfirmedBlock, UiConfirmedBlock};

pub type RawBlock =
	solana_client::rpc_response::Response<solana_client::rpc_response::RpcBlockUpdate>;

pub struct ProcessedBlock {
	pub previous_blockhash: String,
	pub blockhash: String,
	pub slot: i64,
	pub parent_slot: i64,
	pub block_time: i64,
	pub block_height: i64,
	pub transactions: Vec<String>,
}

impl TryFrom<&RawBlock> for ProcessedBlock {
	type Error = AppError;

	fn try_from(value: &RawBlock) -> Result<Self> {
		let block = if let Some(block) = &value.value.block {
			block
		} else {
			error!("No block data");
			return Err(AppError::BlockProcessingError);
		};

		let signatures = if let Some(sig) = &block.signatures {
			sig
		} else {
			error!("No signatures");
			return Err(AppError::BlockProcessingError);
		};

		let block_time = if let Some(block_time) = block.block_time {
			block_time
		} else {
			0
		};

		let block_height = if let Some(block_height) = block.block_height {
			block_height as i64
		} else {
			0
		};

		Ok(Self {
			previous_blockhash: block.previous_blockhash.clone(),
			blockhash: block.blockhash.clone(),
			slot: value.context.slot as i64,
			parent_slot: block.parent_slot as i64,
			block_time,
			block_height,
			transactions: signatures.to_vec(),
		})
	}
}

impl TryFrom<&EncodedConfirmedBlock> for ProcessedBlock {
	type Error = AppError;

	fn try_from(value: &EncodedConfirmedBlock) -> Result<Self> {
		let block_time = if let Some(block_time) = value.block_time {
			block_time
		} else {
			0
		};

		let block_height = if let Some(block_height) = value.block_height {
			block_height as i64
		} else {
			0
		};

		Ok(Self {
			previous_blockhash: value.previous_blockhash.clone(),
			blockhash: value.blockhash.clone(),
			slot: 0,
			parent_slot: value.parent_slot as i64,
			block_time,
			block_height,
			transactions: Vec::new(),
		})
	}
}

impl TryFrom<UiConfirmedBlock> for ProcessedBlock {
	type Error = AppError;

	fn try_from(value: UiConfirmedBlock) -> Result<Self> {
		let block_time = if let Some(block_time) = value.block_time {
			block_time
		} else {
			0
		};

		let block_height = if let Some(block_height) = value.block_height {
			block_height as i64
		} else {
			0
		};

		Ok(Self {
			previous_blockhash: value.previous_blockhash.clone(),
			blockhash: value.blockhash.clone(),
			slot: 0,
			parent_slot: value.parent_slot as i64,
			block_time,
			block_height,
			transactions: value.signatures.clone().unwrap_or(Vec::new()),
		})
	}
}
