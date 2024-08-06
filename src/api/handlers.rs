use {
	axum::{
		extract::{Json, Query, State},
		http::StatusCode,
		response::IntoResponse,
	},
	serde::{Deserialize, Serialize},
	std::sync::Arc,
};

#[derive(Serialize, Deserialize)]
pub struct TransactionQueryParams {
	signature: String,
	start_date: Option<String>,
	end_date: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountQueryParams {
	pubkey: String,
	start_date: Option<String>,
	end_date: Option<String>,
}

pub(crate) async fn get_transaction_handler(
	Query(_params): Query<TransactionQueryParams>,
) -> impl IntoResponse {
	unimplemented!();

	//(StatusCode::OK, Json(response))
}
pub(crate) async fn get_account_handler(
	Query(_params): Query<AccountQueryParams>,
) -> impl IntoResponse {
	unimplemented!();

	//(StatusCode::OK, Json(response))
}
