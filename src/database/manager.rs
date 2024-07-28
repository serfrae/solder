use crate::config::DatabaseConfig;
use crate::error::{AppError, Result};
use crate::models::TransactionRaw;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::{info,error};
use tokio::sync::mpsc;
use tokio_postgres::{Config, NoTls};
use std::time::Duration;
use tokio::time::timeout;

pub struct DatabaseManager {
	pub pool: Pool<PostgresConnectionManager<NoTls>>,
	pub storage_rx: mpsc::Receiver<Vec<TransactionRaw>>,
}

impl DatabaseManager {
	pub async fn new(
		config: DatabaseConfig,
		storage_rx: mpsc::Receiver<Vec<TransactionRaw>>,
	) -> Result<Self> {
		let mut db_config = Config::new();
		db_config.user(config.user);
		db_config.password(config.password);
		db_config.host(config.host);
		db_config.port(config.port);
		db_config.dbname(config.db_name);

		let mgr = PostgresConnectionManager::new(db_config, NoTls);

		let pool = Pool::builder()
			.max_size(config.pool_size)
			.build(mgr)
			.await?;

		Ok(Self { pool, storage_rx })
	}
	pub async fn run_writer(&mut self) -> Result<()> {
		info!("Running writer");
		println!("Running writer"); // Debug print

		info!("Attempting to get database connection");
		println!("Attempting to get database connection"); // Debug print

		let conn = match timeout(Duration::from_secs(5), self.pool.get()).await {
			Ok(Ok(conn)) => {
				info!("Successfully acquired database connection");
				println!("Successfully acquired database connection"); // Debug print
				conn
			}
			Ok(Err(e)) => {
				error!("Failed to get database connection: {:?}", e);
				println!("Failed to get database connection: {:?}", e); // Debug print
				return Err(AppError::DbPoolError(e));
			}
			Err(_) => {
				error!("Timeout while trying to get database connection");
				println!("Timeout while trying to get database connection"); // Debug print
				return Err(AppError::Unknown("Connection timeout".into()));
			}
		};

		info!("Database Manager: Ready to receive transactions");
		println!("Database Manager: Ready to receive transactions"); // Debug print

		while let Some(data) = self.storage_rx.recv().await {
			info!("Received transaction batch. Size: {}", data.len());
			println!("Received transaction batch. Size: {}", data.len()); // Debug print

			let query = "
        INSERT INTO transactions (signature, slot, account_to, account_from, amount, token_type)
            VALUES ($1, $2, $3, $4, $5, $6)
        ";

			for (index, transaction) in data.iter().enumerate() {
				match conn
					.execute(
						query,
						&[
							&transaction.signature,
							&transaction.slot,
							&transaction.account_to,
							&transaction.account_from,
							&transaction.amount,
							&transaction.token_type,
						],
					)
					.await
				{
					Ok(_) => {
						if index % 100 == 0 {
							// Log every 100 transactions
							info!("Successfully wrote transaction {}", index);
							println!("Successfully wrote transaction {}", index); // Debug print
						}
					}
					Err(e) => {
						error!("Error writing transaction {}: {:?}", index, e);
						println!("Error writing transaction {}: {:?}", index, e); // Debug print
						return Err(AppError::DatabaseError(e));
					}
				}
			}
			info!("Transaction batch written to database");
			println!("Transaction batch written to database"); // Debug print
		}

		info!("Writer loop ended");
		println!("Writer loop ended"); // Debug print
		Ok(())
	}
}
