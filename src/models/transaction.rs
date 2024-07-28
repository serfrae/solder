use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionTokenBalance};

pub type RawTransaction = EncodedTransactionWithStatusMeta;
pub type RawTokenBalance = UiTransactionTokenBalance;
pub type ProcessedTransactions = Vec<ProcessedTransaction>;
pub type TokenBalances = Vec<TokenBalance>;

pub struct TransactionWithSig {
	pub signature: String,
	pub transaction: RawTransaction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedTransaction {
	signature: String,
	fee: i64,
	pre_balances: Vec<i64>,
	post_balances: Vec<i64>,
	pre_token_balances: TokenBalances,
	post_token_balances: TokenBalances,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenBalance {
	mint: String,
	amount: i64,
	decimals: i64,
	owner: String,
	program_id: String,
}

impl TryFrom<TransactionWithSig> for ProcessedTransaction {
	type Error = AppError;

	fn try_from(value: TransactionWithSig) -> Result<Self> {
		let TransactionWithSig {
			transaction,
			signature,
		} = value;

		let meta = transaction.meta.ok_or(AppError::NoData);

		let pre_token_balances: TokenBalances = meta
			.pre_token_balances
			.ok_or(AppError::NoData)?
			.iter()
			.map(TokenBalance::from)
			.collect();

		if pre_token_balances.is_empty() {
			return Err(AppError::NoData);
		}

		let post_token_balances: TokenBalances = meta
			.post_token_balances
			.ok_or(AppError::NoData)?
			.iter()
			.map(TokenBalance::from)
			.collect();

		if post_token_balances.is_empty() {
			return Err(AppError::NoData);
		}

		Ok(Self {
			signature,
			fee: meta.fee,
			pre_balances: transaction.meta.pre_balances,
			post_balances: transaction.meta.post_balances,
			pre_token_balances,
			post_token_balances,
		})
	}
}

impl From<RawTokenBalance> for TokenBalance {
	fn from(value: RawTokenBalance) -> Self {
		let decimals = value.ui_token_amount.decimals as i64;
		let amount = if let Some(amount) = value.ui_token_amount.amount {
			amount * 10_usize.pow(decimals)
		} else {
			0
		} as i64;

		Self {
			mint: value.mint,
			owner: value.owner,
			amount,
			decimals,
			program_id: value.program_id,
		}
	}
}
