use crate::database::DatabasePool;
use crate::error::Result;
use std::future::Future;
use std::pin::Pin;

pub trait Storable: Sized + Send {
	fn store(
		self,
		db_pool: DatabasePool,
	) -> Result<Pin<Box<dyn Future<Output = Result<()>> + Send>>>;
}
