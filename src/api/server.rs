use {
	crate::api::handlers::*,
	anyhow::{anyhow, Result},
	axum::{
		http::{
			header::{ACCEPT_ENCODING, AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE},
			Method,
		},
		routing::get,
		Router,
	},
	tokio::net::TcpListener,
	tower_http::cors::{Any, CorsLayer},
};

pub struct Server {
	app: Router,
	listener: TcpListener,
}

impl Server {
	pub async fn new(
		port: u16,
	) -> Self {
		let cors = CorsLayer::new()
			.allow_methods([Method::GET, Method::POST, Method::OPTIONS])
			.allow_headers([
				CONTENT_TYPE,
				AUTHORIZATION,
				CONTENT_ENCODING,
				ACCEPT_ENCODING,
			])
			.allow_origin(Any);


		let app = Router::new()
			.route("/transaction", get(get_transaction_handler))
			.route("/account", get(get_account_handler))
			.layer(cors);

		let addr = format!("0.0.0.0:{}", port);
		let listener = TcpListener::bind(&addr).await.unwrap();

		Server { app, listener }
	}

	pub async fn run(self) -> Result<()> {
		axum::serve(self.listener, self.app)
			.await
			.map_err(|e| anyhow!("Could not start server: {}", e))
	}
}
