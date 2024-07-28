use crate::error::{AppError, Result};
use log::error;

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

impl TryFrom<RawBlock> for ProcessedBlock {
    type Error = AppError;

	fn try_from(value: RawBlock) -> Result<Self> {
		let block = if let Some(block) = value.value.block {
			block
		} else {
			error!("No block data");
			return Err(AppError::BlockProcessingError);
		};

		let signatures = if let Some(sig) = block.signatures {
			sig
		} else {
			error!("No signatures");
			return Err(AppError::BlockProcessingError);
		};

		Ok(Self {
			previous_blockhash: block.previous_blockhash,
			blockhash: block.blockhash,
			slot: value.slot,
			parent_slot: block.parent_slot,
			block_time: block.block_time,
			block_height: block.block_height,
			transactions: signatures,
		})
	}
}
