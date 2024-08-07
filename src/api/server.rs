use {
	crate::{api::handlers::*, database::DatabasePool},
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
        conn_pool: DatabasePool,
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
			.route("/api/transaction", get(get_transaction_handler))
			.route("/api/account", get(get_account_handler))
            .route("/api/block", get(get_block_handler))
			.layer(cors)
            .with_state(conn_pool);

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
