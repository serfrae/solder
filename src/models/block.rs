use solana_client::rpc_response::SlotInfo;
use solana_transaction_status::UiConfirmedBlock;

pub type BlockUpdate =
	solana_client::rpc_response::Response<solana_client::rpc_response::RpcBlockUpdate>;

#[derive(Debug, Clone)]
pub struct ProcessedBlock {
	pub previous_blockhash: String,
	pub blockhash: String,
	pub slot: i64,
	pub parent_slot: i64,
	pub block_time: i64,
	pub block_height: i64,
}

impl From<(SlotInfo, UiConfirmedBlock)> for ProcessedBlock {
	fn from(value: (SlotInfo, UiConfirmedBlock)) -> Self {
		let block_time = if let Some(block_time) = value.1.block_time {
			block_time
		} else {
			0
		};

		let block_height = if let Some(block_height) = value.1.block_height {
			block_height as i64
		} else {
			0
		};

		Self {
			previous_blockhash: value.1.previous_blockhash.clone(),
			blockhash: value.1.blockhash.clone(),
			slot: value.0.root as i64,
			parent_slot: value.1.parent_slot as i64,
			block_time,
			block_height,
		}
	}
}
