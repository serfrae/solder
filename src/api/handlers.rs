use {
	crate::{api::query::*, api::server::AppState, database::DatabasePool},
	axum::{
		extract::{Query, State},
		http::StatusCode,
		response::IntoResponse,
		Json,
	},
	serde::Serialize,
	serde_json::{json, Value},
	std::sync::Arc,
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

fn test() -> impl IntoResponse {
	let mut responses: Vec<Aggregate> = Vec::new();
	responses.push(Aggregate {
		blockhash: "asdads".to_string(),
		slot: 123,
		signature: "12312".to_string(),
		account: "asdasda".to_string(),
		block_time: 1231231,
	});
    (StatusCode::OK, Json(json!(responses)))
}

pub async fn transaction_handler(
	State(_state): State<Arc<AppState>>,
	Query(_params): Query<TransactionQueryParams>,
) -> impl IntoResponse {
	let mut responses: Vec<Aggregate> = Vec::new();
	responses.push(Aggregate {
		blockhash: "asdads".to_string(),
		slot: 123,
		signature: "12312".to_string(),
		account: "asdasda".to_string(),
		block_time: 1231231,
	});
	if responses.is_empty() {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"error": "asodasldh"})),
		)
	} else {
		(StatusCode::OK, Json(json!(responses)))
	}
	//handle_query(pool, params).await
}

pub async fn account_handler(
	State(state): State<Arc<AppState>>,
	Query(params): Query<AccountQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
	handle_query(state.pool.clone(), params).await
}

pub async fn block_handler(
	State(state): State<Arc<AppState>>,
	Query(params): Query<BlockQueryParams>,
) -> impl IntoResponse {
	handle_block_query(state.pool.clone(), params).await
}

async fn handle_query<T>(
	pool: DatabasePool,
	params: T,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)>
where
	T: Into<QueryParams>,
{
	match execute_query(pool, params.into()).await {
		Ok(transactions) => Ok((StatusCode::OK, Json(json!(transactions)))),
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
