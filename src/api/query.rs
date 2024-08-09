use crate::{database::DatabasePool, models::Aggregate};
use axum::{http::StatusCode, Json};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct TransactionQueryParams {
	signature: Option<String>,
	from: Option<String>, // YYYY-MM-DD
	to: Option<String>,   // YYYY-MM-DD
}

#[derive(Deserialize)]
pub struct AccountQueryParams {
	pubkey: Option<String>,
	from: Option<String>, // YYYY-MM-DD
	to: Option<String>,   // YYYY-MM-DD
}

#[derive(Deserialize)]
pub struct BlockQueryParams {
	hash: Option<String>,
	slot: Option<i64>,
}

pub enum QueryParams {
	Transaction(TransactionQueryParams),
	Account(AccountQueryParams),
}

impl From<TransactionQueryParams> for QueryParams {
	fn from(params: TransactionQueryParams) -> Self {
		QueryParams::Transaction(params)
	}
}

impl From<AccountQueryParams> for QueryParams {
	fn from(params: AccountQueryParams) -> Self {
		QueryParams::Account(params)
	}
}

pub async fn execute_query(
	pool: DatabasePool,
	params: QueryParams,
) -> Result<Vec<Aggregate>, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("{}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to get database connection: {}", e)})),
		)
	})?;

	let (base_query, query_params) = build_query(params);

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = query_params
		.iter()
		.map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
		.collect();

	let rows = client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("{}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"error": format!("Failed to get database connection: {}", e)})),
			)
		})?;

	let transactions: Vec<Aggregate> = rows
		.into_iter()
		.map(|row| Aggregate {
			blockhash: row.get("blockhash"),
			slot: row.get("slot"),
			block_time: row.get("block_time"),
			signature: row.get("signature"),
			account: row.get("account"),
		})
		.collect();

	Ok(transactions)
}

pub async fn execute_block_query(
	pool: DatabasePool,
	params: BlockQueryParams,
) -> Result<Vec<Aggregate>, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("{}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to get database connection: {}", e)})),
		)
	})?;

	let (base_query, query_params) = build_block_query(params);

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = query_params
		.iter()
		.map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
		.collect();

	let rows = client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("{}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "error": format!("Error querying transactions: {}", e) })),
			)
		})?;

	let transactions: Vec<Aggregate> = rows
		.into_iter()
		.map(|row| Aggregate {
			blockhash: row.get("blockhash"),
			slot: row.get("slot"),
			block_time: row.get("block_time"),
			signature: row.get("signature"),
			account: row.get("account"),
		})
		.collect();

	Ok(transactions)
}

fn build_query(
	params: QueryParams,
) -> (
	String,
	Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
) {
	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

	match params {
		QueryParams::Transaction(params) => {
			if let Some(signature) = &params.signature {
				base_query.push_str(" signature = $1");
				query_params.push(Box::new(signature.clone()));
			} else {
				add_date_conditions(&mut base_query, &mut query_params, &params.from, &params.to);
			}
		}
		QueryParams::Account(params) => {
			if let Some(pubkey) = &params.pubkey {
				base_query.push_str(" account = $1");
				query_params.push(Box::new(pubkey.clone()));
			} else {
				add_date_conditions(&mut base_query, &mut query_params, &params.from, &params.to);
			}
		}
	}

	if query_params.is_empty() {
		base_query.push_str(" TRUE");
	}

	base_query.push_str(" ORDER BY slot ASC");

	(base_query, query_params)
}

fn build_block_query(
	params: BlockQueryParams,
) -> (
	String,
	Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
) {
	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

	if let Some(hash) = &params.hash {
		base_query.push_str(" blockhash = $1");
		query_params.push(Box::new(hash.clone()));
	} else if let Some(slot) = &params.slot {
		base_query.push_str(" slot = $1");
		query_params.push(Box::new(*slot));
	} else {
		base_query.push_str(" FALSE");
	}

	base_query.push_str(" ORDER BY slot ASC");

	(base_query, query_params)
}

fn add_date_conditions(
	base_query: &mut String,
	query_params: &mut Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
	from: &Option<String>,
	to: &Option<String>,
) {
	if let Some(from) = parse_date_opt(from) {
		base_query.push_str(" block_time >= $1");
		query_params.push(Box::new(from));
	}
	if let Some(to) = parse_date_opt(to) {
		if !query_params.is_empty() {
			base_query.push_str(" AND");
		}
		base_query.push_str(" block_time <= $2");
		query_params.push(Box::new(to));
	}
}

fn parse_date_opt(date_str: &Option<String>) -> Option<i64> {
	if let Some(date_str) = date_str {
		let naive_date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
		let datetime = Utc.from_utc_datetime(&naive_date).timestamp();
		Some(datetime)
	} else {
		None
	}
}
