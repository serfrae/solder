pub mod aggregate;
pub mod log;

pub use aggregate::*;
pub use log::*;

use crate::error::{AppError, Result};
use solana_transaction_status::{
	parse_accounts::ParsedAccount, EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage,
	UiTransaction,
};

pub type BlockUpdate =
	solana_client::rpc_response::Response<solana_client::rpc_response::RpcBlockUpdate>;

pub type AccountKeys = Vec<ParsedAccount>;

pub trait TryDecode<T>: Sized {
	type Error;
	fn try_decode(value: T) -> Result<Self>;
}

pub trait FromMsg<T> {
	fn from_msg(msg: T) -> Self;
}

impl TryDecode<EncodedTransactionWithStatusMeta> for UiTransaction {
	type Error = AppError;
	fn try_decode(value: EncodedTransactionWithStatusMeta) -> Result<Self> {
		if let EncodedTransaction::Json(tx) = value.transaction {
			Some(tx)
		} else {
			None
		}
		.ok_or(AppError::NoDecodedTransaction)
	}
}

impl FromMsg<UiMessage> for Vec<String> {
	fn from_msg(msg: UiMessage) -> Self {
		match msg {
			UiMessage::Raw(msg) => msg.account_keys,
			UiMessage::Parsed(msg) => msg.account_keys.into_iter().map(|x| x.pubkey).collect(),
		}
	}
}
