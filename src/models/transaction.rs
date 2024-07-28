use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
	SOL,
	Token,
}

impl fmt::Display for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let s = match self {
			TokenType::SOL => "SOL",
			TokenType::Token => "Token",
		};
		write!(f, "{}", s)
	}
}

impl FromStr for TokenType {
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		match s {
			"SOL" => Ok(TokenType::SOL),
			"Token" => Ok(TokenType::Token),
			_ => Err(AppError::ParseTokenType),
		}
	}

	type Err = AppError;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRaw {
	pub signature: String,
	pub slot: i64,
	pub account_from: String,
	pub account_to: String,
	pub amount: i64,
	pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
	pub signature: Signature,
	pub slot: u64,
	pub account_from: Pubkey,
	pub account_to: Pubkey,
	pub amount: u64,
	pub token_type: TokenType,
}

impl Transaction {
	pub fn to_row(&self) -> (String, i64, String, String, i64, String) {
		(
			self.signature.to_string(),
			self.slot as i64,
			self.account_from.to_string(),
			self.account_to.to_string(),
			self.amount as i64,
			self.token_type.to_string(),
		)
	}

	pub fn from_row(row: &tokio_postgres::Row) -> Result<Self> {
		Ok(Self {
			signature: Signature::from_str(row.get("signature"))?,
			slot: row.get::<_, i64>("slot") as u64,
			account_from: Pubkey::from_str(row.get("account_from"))?,
			account_to: Pubkey::from_str(row.get("account_to"))?,
			amount: row.get::<_, i64>("amount") as u64,
			token_type: TokenType::from_str(row.get("token_type"))?,
		})
	}
}
