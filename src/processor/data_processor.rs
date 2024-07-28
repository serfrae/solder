use crate::error::Result;
use crate::models::{Logs, TokenType, TransactionRaw};
use std::str::FromStr;
use log::info;

pub struct Processor;

impl Processor {
	pub fn process_log(data: Logs) -> Result<Vec<TransactionRaw>> {
		let mut transactions = Vec::new();
        info!("Signature: {}", data.value.signature);
        info!("Slot: {}", data.context.slot);

		for line in data.value.logs {
            info!("{}", line);
			let strings: Vec<&str> = line.split_whitespace().collect();
			if strings.contains(&"SOL") {
				let amount = i64::from_str(strings[1])?;

				transactions.push(TransactionRaw {
					signature: data.value.signature.clone(),
					slot: data.context.slot as i64,
					account_from: strings[4].to_string(),
					account_to: strings[6].to_string(),
					amount,
					token_type: TokenType::SOL.to_string(),
				});
			} else if strings.contains(&"tokens") {
				let amount = i64::from_str(strings[2])?;
				transactions.push(TransactionRaw {
					signature: data.value.signature.clone(),
					slot: data.context.slot as i64,
					account_from: strings[6].to_string(),
					account_to: strings[8].to_string(),
					amount,
					token_type: TokenType::Token.to_string(),
				});
			} else {
				continue;
			}
		}

		Ok(transactions)
	}
}
