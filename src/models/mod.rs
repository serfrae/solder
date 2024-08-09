pub mod aggregate;

pub use aggregate::*;

use crate::error::{AppError, Result};
use solana_transaction_status::{
	EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage, UiTransaction,
};

/// Unused type alias
pub type BlockUpdate =
	solana_client::rpc_response::Response<solana_client::rpc_response::RpcBlockUpdate>;

/// For decoding `Encoded` types in `solana_transaction_status`
pub trait TryDecode<T>: Sized {
	type Error;
	fn try_decode(value: T) -> Result<Self>;
}

/// For retrieving types from a `solana_transaction_status::Message`
pub trait FromMsg<T> {
	fn from_msg(msg: T) -> Self;
}

/// To decoded `EncodedTransactionWithStatusMeta` to a UiTransaction to get signatures and accounts
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

/// Get accounts from a `UiMessage`
impl FromMsg<UiMessage> for Vec<String> {
	fn from_msg(msg: UiMessage) -> Self {
		match msg {
			UiMessage::Raw(msg) => msg.account_keys,
			UiMessage::Parsed(msg) => msg.account_keys.into_iter().map(|x| x.pubkey).collect(),
		}
	}
}
