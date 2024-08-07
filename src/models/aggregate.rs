use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Aggregate {
	pub blockhash: String,
	pub slot: i64,
	pub block_time: i64,
	pub signature: String,
	pub account: String,
}
