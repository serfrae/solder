use super::Storable;
use crate::{
	database::DatabasePool,
	error::Result,
	models::ProcessedTransactionLogs,
};
use std::{future::Future, pin::Pin};

impl Storable for ProcessedTransactionLogs {
	fn store(
		self,
		_db_pool: DatabasePool,
	) -> Result<Pin<Box<dyn Future<Output = Result<()>> + Send>>> {
        unimplemented!();
		//let db_pool_clone = db_pool.clone();

		//Ok(Box::pin(async move {
		//	let (transaction_task, block_task) = tokio::join!(
		//		tokio::task::spawn_local(async move {
		//			//let conn = db_pool_clone.get().await.map_err(AppError::DbPoolError)?;

		//			Ok::<(), AppError>(())
		//		}),
		//		tokio::task::spawn_local(async move {
		//			//let conn = db_pool.get().await?;

		//			Ok::<(), AppError>(())
		//		})
		//	);

		//	transaction_task??;
		//	block_task??;

		//	Ok(())
		//}))
	}
}
