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
pub struct ProcessedTransactionWithSig {
	signature: String,
	transaction: ProcessedTransaction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedTransaction {
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

impl TryFrom<RawTransaction> for ProcessedTransaction {
	type Error = AppError;
	fn try_from(value: RawTransaction) -> Result<Self> {
		let meta = value.meta.ok_or(AppError::NoData)?;

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

		let pre_balances = meta.pre_balances.into_iter().map(|x| x as i64).collect();
		let post_balances = meta.post_balances.into_iter().map(|x| x as i64).collect();

		Ok(Self {
			fee: meta.fee as i64,
			pre_balances,
			post_balances,
			pre_token_balances,
			post_token_balances,
		})
	}
}

impl TryFrom<TransactionWithSig> for ProcessedTransactionWithSig {
	type Error = AppError;

	fn try_from(value: TransactionWithSig) -> Result<Self> {
		let TransactionWithSig {
			transaction,
			signature,
		} = value;

		let processed_transaction = ProcessedTransaction::try_from(transaction)?;

		Ok(Self {
			signature,
			transaction: processed_transaction, 
		})
	}
}

impl From<&RawTokenBalance> for TokenBalance {
	fn from(value: &RawTokenBalance) -> Self {
		let decimals = value.ui_token_amount.decimals;
		let amount = if let Some(amount) = value.ui_token_amount.ui_amount {
			amount * 10f64.powf(decimals as f64)
		} else {
			0f64
		} as i64;

		Self {
			mint: value.mint.clone(),
			owner: value.owner.clone().unwrap_or("".to_string()),
			amount,
			decimals: decimals as i64,
			program_id: value.program_id.clone().unwrap_or("".to_string()),
		}
	}
}
