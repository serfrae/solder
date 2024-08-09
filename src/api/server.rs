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
    std::sync::Arc,
};

pub struct Server {
	app: Router,
	listener: TcpListener,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: DatabasePool,
}

impl Server {
	pub async fn new(conn_pool: DatabasePool, port: u16) -> Self {
		let cors = CorsLayer::new()
			.allow_methods([Method::GET, Method::OPTIONS])
			.allow_headers([
				CONTENT_TYPE,
				AUTHORIZATION,
				CONTENT_ENCODING,
				ACCEPT_ENCODING,
			])
			.allow_origin(Any);

        let state = Arc::new(AppState { pool: conn_pool });

		let app = Router::new()
            .route("/", get(root))
			.route("/api/transaction", get(transaction_handler))
			.route("/api/account", get(account_handler))
			.route("/api/block", get(block_handler))
			.fallback(handler_404)
			.layer(cors)
			.with_state(state);

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
