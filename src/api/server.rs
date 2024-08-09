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

/// Axum server, axum uses its own worker pool so no need to implement our own here.
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

		let app = Router::new()
			.route("/", get(root))
			.route("/api/transaction/:signature", get(transaction_handler))
			.route("/api/account/:pubkey", get(account_handler))
			.route("/api/block/:blockhash", get(block_handler))
            .route("/api/slot/:slot_number", get(slot_handler))
			.fallback(handler_404)
			.layer(cors)
			.with_state(conn_pool);

		let addr = format!("0.0.0.0:{}", port);
		let listener = TcpListener::bind(&addr).await.unwrap();

		Server { app, listener }
	}

    /// Run with a ctrl+c shutdown signal for graceful shutdowns
	pub async fn run(self) -> Result<()> {
		let shutdown_signal = async {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to install Ctrl+C handler");
			log::info!("\nReceived shutdown signal. Shutting down gracefully...");
		};

		axum::serve(self.listener, self.app)
			.with_graceful_shutdown(shutdown_signal)
			.await
			.map_err(|e| anyhow!("Could not start server: {}", e))?;

		log::info!("Server has shutdown");

		Ok(())
	}
}
