use {
	crate::{
		api::query::*,
		database::DatabasePool,
		models::{Aggregate, BlockResponse, BlockTransactions, TransactionResponse},
	},
	axum::{
		extract::{Path, Query, State},
		http::StatusCode,
		response::IntoResponse,
		Json,
	},
    std::collections::HashMap,
	serde_json::Value,
	tokio_postgres::row::Row,
};

/// Simple call to ensure server is running
pub(crate) async fn root() -> &'static str {
	"Server is running!"
}

/// Fallback for malformed requests
pub(crate) async fn handler_404() -> impl IntoResponse {
	(
		StatusCode::NOT_FOUND,
		"The requested resource was not found",
	)
}

/// `/api/accounts/:pubkey?to=YYYY-MM-DD&from-YYYY-MM-DD`, takes a connection pool
/// to the database as a state parameter for data retrieval will return 
/// `(StatusCode::ERROR_TYPE, description)` if not `Ok()`.
///
/// Parameters:
/// `to: Option<String>`,
/// `from: Option<String>`.
pub async fn account_handler(
	State(pool): State<DatabasePool>,
	Path(pubkey): Path<String>,
	Query(params): Query<AccountQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	match handle_query(
		pool.clone(),
		QueryType::Account {
			pubkey,
			from: params.from,
			to: params.to,
		},
	)
	.await
	{
		Ok(rows) => Ok(build_account_response(rows)),
		Err(e) => Err(e),
	}
}

/// `/api/transaction/:signature`, takes a connection pool to the database as a state parameter
/// for data retrieval will return `(StatusCode::ERROR_TYPE, description)` if not `Ok()`.
pub async fn transaction_handler(
	State(pool): State<DatabasePool>,
	Path(signature): Path<String>,
) -> impl IntoResponse {
	match handle_query(pool.clone(), QueryType::Transaction(signature)).await {
		Ok(rows) => Ok(build_transaction_response(rows)),
		Err(e) => Err(e),
	}
}

/// `/api/block/:blockhash`, takes connection pool to the database as a state parameter
/// for data retrieval will return `(StatusCode::ERROR_TYPE, description)` if not `Ok()`.
pub async fn block_handler(
	State(pool): State<DatabasePool>,
	Path(blockhash): Path<String>,
) -> impl IntoResponse {
	match handle_query(pool.clone(), QueryType::Block(blockhash)).await {
		Ok(rows) => Ok(build_block_response(rows)),
		Err(e) => Err(e),
	}
}

/// `/api/block/:slot_number`, takes connection pool to the database as a state parameter
/// for data retrieval will return `(StatusCode::ERROR_TYPE, description)` if not `Ok()`.
pub async fn slot_handler(
	State(pool): State<DatabasePool>,
	Path(slot_number): Path<i64>,
) -> impl IntoResponse {
	match handle_query(pool.clone(), QueryType::Slot(slot_number)).await {
		Ok(rows) => Ok(build_slot_response(rows)),
		Err(e) => Err(e),
	}
}

/// Query handler
async fn handle_query(
	pool: DatabasePool,
	query_type: QueryType,
) -> Result<Vec<Row>, (StatusCode, Json<Value>)> {
	execute_query(pool, query_type).await
}

/// Builds an `axum::response::Response` from a `Vec<Row>` should never throw an error.
fn build_account_response(rows: Vec<Row>) -> impl IntoResponse {
	let aggregate: Vec<Aggregate> = rows
		.into_iter()
		.map(|row| Aggregate {
			blockhash: row.get("blockhash"),
			slot: row.get("slot"),
			block_time: row.get("block_time"),
			signature: row.get("signature"),
			account: row.get("account"),
		})
		.collect();
	(StatusCode::OK, Json(aggregate))
}

/// builds an `axum::response::response` from a `vec<row>` should never throw an error.
fn build_transaction_response(rows: Vec<Row>) -> impl IntoResponse {
	let first_row = &rows[0];
	(
		StatusCode::OK,
		Json(TransactionResponse {
			blockhash: first_row.get("blockhash"),
			slot: first_row.get("slot"),
			block_time: first_row.get("block_time"),
			signature: first_row.get("signature"),
			accounts: rows
				.into_iter()
				.map(|row| row.get("account"))
				.collect::<Vec<String>>(),
		}),
	)
}
/// Builds an `axum::response::Response` from a `Vec<Row>` should never throw an error.
/// Uses a Hashmap to avoid cloning the vector, should keep lookups to O(1).
/// Had previously used Itertools::chunk_by but required cloning the Vec.
fn build_block_response(rows: Vec<Row>) -> impl IntoResponse {
	let mut transaction_map: HashMap<String, Vec<String>> = HashMap::new();
	let mut first_row = None;

	for row in rows {
		if first_row.is_none() {
			first_row = Some(row.clone());
		}
		let signature: String = row.get("signature");
		let account: String = row.get("account");
		transaction_map
			.entry(signature)
			.or_insert_with(Vec::new)
			.push(account);
	}

	let transactions: Vec<BlockTransactions> = transaction_map
		.into_iter()
		.map(|(signature, accounts)| BlockTransactions {
			signature,
			accounts,
		})
		.collect();

	let first_row = first_row.unwrap();
	(
		StatusCode::OK,
		Json(BlockResponse {
			blockhash: first_row.get("blockhash"),
			slot: first_row.get("slot"),
			block_time: first_row.get("block_time"),
			transactions,
		}),
	)
}

/// Alias for `build_block_response`
fn build_slot_response(rows: Vec<Row>) -> impl IntoResponse {
	build_block_response(rows)
}
