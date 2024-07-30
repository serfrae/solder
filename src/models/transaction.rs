use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use solana_transaction_status::{
	EncodedTransactionWithStatusMeta, UiTransactionTokenBalance,
};

pub struct TransactionWithSignature {
	pub signature: String,
	pub transaction: EncodedTransactionWithStatusMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedTransaction {
    signature: Option<String>,
	fee: i64,
	pre_balances: Vec<i64>,
	post_balances: Vec<i64>,
	pre_token_balances: Vec<TokenBalance>,
	post_token_balances: Vec<TokenBalance>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenBalance {
	mint: String,
	amount: i64,
	decimals: i64,
	owner: String,
	program_id: String,
}

impl TryFrom<EncodedTransactionWithStatusMeta> for ProcessedTransaction {
	type Error = AppError;
	fn try_from(value: EncodedTransactionWithStatusMeta) -> Result<Self> {
		let meta = value.meta.ok_or(AppError::NoData)?;

		let pre_token_balances: Vec<TokenBalance> = meta
			.pre_token_balances
			.ok_or(AppError::NoData)?
			.iter()
			.map(TokenBalance::from)
			.collect();

		let post_token_balances: Vec<TokenBalance> = meta
			.post_token_balances
			.ok_or(AppError::NoData)?
			.iter()
			.map(TokenBalance::from)
			.collect();

		let pre_balances = meta.pre_balances.into_iter().map(|x| x as i64).collect();
		let post_balances = meta.post_balances.into_iter().map(|x| x as i64).collect();

		Ok(Self {
            signature: None,
			fee: meta.fee as i64,
			pre_balances,
			post_balances,
			pre_token_balances,
			post_token_balances,
		})
	}
}

impl From<&UiTransactionTokenBalance> for TokenBalance {
	fn from(value: &UiTransactionTokenBalance) -> Self {
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
