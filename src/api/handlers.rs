use {
	crate::{api::query::*, database::DatabasePool},
	axum::{
		extract::{Query, State},
		http::StatusCode,
		response::IntoResponse,
		Json,
	},
	serde::Serialize,
	serde_json::Value,
};

#[derive(Serialize, Clone, Debug)]
pub struct Aggregate {
	pub blockhash: String,
	pub slot: i64,
	pub block_time: i64,
	pub signature: String,
	pub account: String,
}

pub(crate) async fn root() -> &'static str {
	"Server is running!"
}

pub(crate) async fn handler_404() -> impl IntoResponse {
	(
		StatusCode::NOT_FOUND,
		"The requested resource was not found",
	)
}

pub async fn transaction_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<TransactionQueryParams>,
) -> impl IntoResponse {
	handle_query(pool.clone(), params).await
}

pub async fn account_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<AccountQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	handle_query(pool.clone(), params).await
}

pub async fn block_handler(
	State(pool): State<DatabasePool>,
	Query(params): Query<BlockQueryParams>,
) -> impl IntoResponse {
	handle_block_query(pool.clone(), params).await
}

async fn handle_query<T>(
	pool: DatabasePool,
	params: T,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)>
where
	T: Into<QueryParams>,
{
	match execute_query(pool, params.into()).await {
		Ok(transactions) => Ok((StatusCode::OK, Json(transactions))),
		Err(e) => Err(e),
	}
}

async fn handle_block_query(
	pool: DatabasePool,
	params: BlockQueryParams,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	match execute_block_query(pool, params).await {
		Ok(transactions) => Ok((StatusCode::OK, Json(transactions))),
		Err(e) => Err(e),
	}
}
