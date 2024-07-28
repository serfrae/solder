use crate::error::Result;
use bb8::PooledConnection;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub trait Storable: Sized + Send {
	fn store(&self, db_pool: ) -> Result<()>;
}
