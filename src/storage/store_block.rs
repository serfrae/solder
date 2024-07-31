use super::Storable;
use crate::{
	database::DatabasePool,
	error::{AppError, Result},
	models::ProcessedBlockAndTransactions,
};
use std::{future::Future, pin::Pin};

impl Storable for ProcessedBlockAndTransactions {
	fn store(
		&self,
		db_pool: DatabasePool,
	) -> Result<Pin<Box<dyn Future<Output = Result<()>> + Send>>> {
		let db_pool_clone = db_pool.clone();
		let data = self.0.clone();

		Ok(Box::pin(async move {
			let (transaction_task, block_task) = tokio::join!(
				tokio::task::spawn(async move {
					let conn = if let Ok(conn) = db_pool_clone.get().await {
						conn
					} else {
                        log::error!("Connection error: {}", conn?);
						return Err(AppError::DbPoolError);
					};
					log::info!("Obtained connection from connection pool");
					conn.execute(
						"CREATE TABLE IF NOT EXISTS block (
            previous_blockhash TEXT NOT NULL,
            blockhash TEXT NOT NULL PRIMARY KEY,
            slot BIGINT NOT NULL,
            parent_slot BIGINT NOT NULL,
            block_time BIGINT NOT NULL,
            block_height BIGINT NOT NULL
        )",
						&[],
					)
					.await?;

					conn.execute(
						"INSERT INTO block (
                previous_blockhash, 
                blockhash, 
                slot, 
                parent_slot, 
                block_time, 
                block_height
            ) VALUES ($1, $2, $3, $4, $5, $6)",
						&[
							&data.previous_blockhash,
							&data.blockhash,
							&data.slot,
							&data.parent_slot,
							&data.block_time,
							&data.block_height,
						],
					)
					.await?;
					log::info!("[STORAGE] Block: {} stored", data.blockhash);

					Ok::<(), AppError>(())
				}),
				tokio::task::spawn(async move {
					//let conn = db_pool.get().await?;

					Ok::<(), AppError>(())
				})
			);

			transaction_task??;
			block_task??;

			Ok(())
		}))
	}
}
