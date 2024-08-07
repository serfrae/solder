use {
	crate::{database::DatabasePool, models::Aggregate},
	axum::{
		extract::{Json, Query, State},
		http::StatusCode,
		response::IntoResponse,
	},
	chrono::{NaiveDateTime, TimeZone, Utc},
	serde::{Deserialize, Serialize},
	serde_json::{json, Value},
};

#[derive(Deserialize)]
struct TransactionQueryParams {
	signature: Option<String>,
	from: Option<String>, // YYYY-MM-DD
	to: Option<String>,   // YYYY-MM-DD
}

#[derive(Deserialize)]
struct AccountQueryParams {
	pubkey: Option<String>,
	from: Option<String>, // YYYY-MM-DD
	to: Option<String>,   // YYYY-MM-DD
}

#[derive(Deserialize)]
struct BlockQueryParams {
	hash: Option<String>,
	slot: Option<i64>,
}

#[derive(Serialize)]
struct Response {
	transactions: Vec<Aggregate>,
}

pub(crate) async fn get_transaction_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<TransactionQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("Failed to get database connection: {}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to retrieve transactions: {}", e)})),
		)
	})?;

	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

	if let Some(signature) = &params.signature {
		base_query.push_str(" signature = $1");
		query_params.push(Box::new(signature));
	} else {
		if let Some(from) = parse_date_opt(&params.from) {
			base_query.push_str(" slot >= $2");
			query_params.push(Box::new(from));
		}
		if let Some(to) = parse_date_opt(&params.to) {
			if !query_params.is_empty() {
				base_query.push_str(" AND");
			}
			base_query.push_str(" slot <= $3");
			query_params.push(Box::new(to));
		}
	}

	if query_params.is_empty() {
		base_query.push_str(" TRUE"); // No conditions, return all
	}

	base_query.push_str(" ORDER BY slot ASC");

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
		query_params.iter().map(AsRef::as_ref).collect();

	let rows = client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("Error querying transactions: {}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"error": format!("Error retrieving transactions: {}", e)})),
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

	Ok((StatusCode::OK, Json(Response { transactions })))
}
pub(crate) async fn get_account_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<AccountQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("Failed to get database connection: {}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to retrieve transactions: {}", e)})),
		)
	})?;

	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

	if let Some(signature) = &params.pubkey {
		base_query.push_str(" account = $1");
		query_params.push(Box::new(signature));
	} else {
		if let Some(from) = parse_date_opt(&params.from) {
			base_query.push_str(" slot >= $2");
			query_params.push(Box::new(from));
		}
		if let Some(to) = parse_date_opt(&params.to) {
			if !query_params.is_empty() {
				base_query.push_str(" AND");
			}
			base_query.push_str(" slot <= $3");
			query_params.push(Box::new(to));
		}
	}

	if query_params.is_empty() {
		base_query.push_str(" TRUE"); // No conditions, return all
	}

	base_query.push_str(" ORDER BY slot ASC");

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
		query_params.iter().map(AsRef::as_ref).collect();

	let rows = client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("Error querying transactions: {}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"error": format!("Error retrieving transactions: {}", e)})),
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

	Ok((StatusCode::OK, Json(Response { transactions })))
}

pub(crate) async fn get_block_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<BlockQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("Failed to get database connection: {}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to retrieve transactions: {}", e)})),
		)
	})?;

	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();

	if let Some(hash) = &params.hash {
		base_query.push_str(" blockhash = $1");
		query_params.push(Box::new(hash));
	} else if let Some(slot) = &params.slot {
		base_query.push_str(" slot = $1");
		query_params.push(Box::new(slot));
	} else {
		log::error!("Client did not provide a blockhash or slot");
		return Err((
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("No blockhash or slot provided in request")})),
		));
	}

	base_query.push_str(" ORDER BY slot ASC");

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
		query_params.iter().map(AsRef::as_ref).collect();

	let rows = client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("Error querying transactions: {}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"error": format!("Error retrieving transactions: {}", e)})),
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

	Ok((StatusCode::OK, Json(Response { transactions })))
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
