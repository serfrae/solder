use super::Storable;
use crate::{
	database::DatabasePool,
	error::{AppError, Result},
	models::Aggregate,
};
use std::{future::Future, pin::Pin};

impl Storable for Vec<Option<Aggregate>> {
	fn store(
		self,
		db_pool: DatabasePool,
	) -> Result<Pin<Box<dyn Future<Output = Result<()>> + Send>>> {
		let db_pool_clone = db_pool.clone();

		Ok(Box::pin(async move {
			let mut conn = db_pool_clone.get().await.map_err(|e| {
				log::error!("Could not get connection from the pool: {}", e);
				AppError::DbPoolError(e)
			})?;

			log::info!("Obtained connection from connection pool");

			conn.execute(
				"CREATE TABLE IF NOT EXISTS transaction_accounts (
            blockhash TEXT NOT NULL,
            slot BIGINT NOT NULL,
            block_time BIGINT NOT NULL,
            signature TEXT NOT NULL,
            account TEXT NOT NULL,
            PRIMARY KEY (blockhash, signature, account)
        )",
				&[],
			)
			.await
			.map_err(|e| {
				log::error!("Error creating table: {}", e);
				AppError::DatabaseError(e)
			})?;

			let transaction = conn.transaction().await.map_err(|e| {
				log::error!("Error starting transaction: {}", e);
				AppError::DatabaseError(e)
			})?;

			for tx in self {
				if let Some(tx) = tx {
					transaction
						.execute(
							"INSERT INTO transaction_accounts (
                            blockhash, 
                            slot, 
                            block_time, 
                            signature,
                            account
                        ) VALUES ($1, $2, $3, $4, $5)",
							&[
								&tx.blockhash,
								&tx.slot,
								&tx.block_time,
								&tx.signature,
								&tx.account,
							],
						)
						.await
						.map_err(|e| {
							log::error!("Error inserting data: {}", e);
							AppError::DatabaseError(e)
						})?;
				}
			}

			transaction.commit().await.map_err(|e| {
				log::error!("Error committing transaction: {}", e);
				AppError::DatabaseError(e)
			})?;

			Ok(())
		}))
	}
}
