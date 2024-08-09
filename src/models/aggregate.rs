use serde::Serialize;

/// Used as both the storage type and response type
#[derive(Serialize, Clone, Debug)]
pub struct Aggregate {
	pub blockhash: String,
	pub slot: i64,
	pub block_time: i64,
	pub signature: String,
	pub account: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct BlockResponse {
	pub blockhash: String,
	pub slot: i64,
	pub block_time: i64,
	pub transactions: Vec<BlockTransactions>,
}

#[derive(Serialize, Clone, Debug)]
pub struct BlockTransactions {
	pub signature: String,
	pub accounts: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct TransactionResponse {
	pub blockhash: String,
	pub slot: i64,
	pub block_time: i64,
	pub signature: String,
	pub accounts: Vec<String>,
}
