use crate::database::DatabasePool;
use axum::{http::StatusCode, Json};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_postgres::row::Row;

/// Type alias for code cleanliness
type QueryParams = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Enum to define the types of query the server can receive and their respectie parameters
#[derive(Deserialize)]
pub enum QueryType {
	Transaction(String),
	Block(String),
	Slot(i64),
	Account {
		pubkey: String,
		from: Option<String>,
		to: Option<String>,
	},
}

/// Optional parameters for `/api/account/{pubkey}`, `from` and `to` should be provided
/// in YYYY-MM-DD format.
///
/// If `from` is not provided, query will retrieve all data from the beginning of data collection
/// till `to`.
/// If `to` is not provided, query will retrieve all data from `from` till now.
/// If neither parameter is provided, all data regarding an account is retrieved.
#[derive(Deserialize)]
pub struct AccountQueryParams {
	pub from: Option<String>, // YYYY-MM-DD
	pub to: Option<String>,   // YYYY-MM-DD
}

/// Builds and executes a database query
pub async fn execute_query(
	pool: DatabasePool,
	query_type: QueryType,
) -> Result<Vec<Row>, (StatusCode, Json<Value>)> {
	let client = pool.get().await.map_err(|e| {
		log::error!("{}", e);
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": format!("Failed to get database connection: {}", e)})),
		)
	})?;

	let (base_query, query_params) = build_query(query_type);

	let query_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = query_params
		.iter()
		.map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
		.collect();

	Ok(client
		.query(&base_query, &query_refs[..])
		.await
		.map_err(|e| {
			log::error!("{}", e);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"error": format!("Failed to get database connection: {}", e)})),
			)
		}))?
}

/// Build database query from query type, for types `QueryType::Transaction`, `QueryType::Slot`,
/// and `QueryType::Block` the query is built solely from the Path. For `QueryType::Account` 
/// additional time parameters may be passed in. TODO: Pagination
fn build_query(query_type: QueryType) -> (String, Vec<QueryParams>) {
	let mut base_query = "SELECT * FROM transaction_accounts WHERE".to_string();
	let mut query_params: Vec<QueryParams> = Vec::new();

	match query_type {
		QueryType::Transaction(signature) => {
			base_query.push_str(" signature = $1");
			query_params.push(Box::new(signature) as QueryParams);
		}
		QueryType::Slot(slot) => {
			base_query.push_str(" slot = $1");
			query_params.push(Box::new(slot) as QueryParams);
		}
		QueryType::Block(blockhash) => {
			base_query.push_str(" blockhash = $1");
			query_params.push(Box::new(blockhash) as QueryParams);
		}
		QueryType::Account { pubkey, from, to } => {
			base_query.push_str(" account = $1");
			query_params.push(Box::new(pubkey) as QueryParams);
			add_date_conditions(&mut base_query, &mut query_params, &from, &to);
		}
	}

	base_query.push_str(" ORDER BY signature ASC");

	(base_query, query_params)
}

/// Converts a String date of format YYYY-MM-DD to Unix time and inserts it into a query
/// currently only supported for accounts
fn add_date_conditions(
	base_query: &mut String,
	query_params: &mut Vec<QueryParams>,
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

/// Parses optional date
fn parse_date_opt(date_str: &Option<String>) -> Option<i64> {
	if let Some(date_str) = date_str {
		let naive_date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
		let datetime = Utc.from_utc_datetime(&naive_date).timestamp();
		Some(datetime)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
    
}
