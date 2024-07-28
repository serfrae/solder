use crate::{config::DatabaseConfig, error::Result};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::sync::Arc;
use tokio_postgres::{Config, NoTls};

pub type DatabasePool = Arc<Pool<PostgresConnectionManager<NoTls>>>;

pub async fn create_database_pool(config: &DatabaseConfig) -> Result<DatabasePool> {
	let mut db_config = Config::new();
	db_config
		.user(&config.user)
		.password(&config.password)
		.host(&config.host)
		.port(config.port)
		.dbname(&config.db_name);

	let mgr = PostgresConnectionManager::new(db_config, NoTls);

	let pool = Pool::builder()
		.max_size(config.pool_size)
		.build(mgr)
		.await?;

	Ok(Arc::new(pool))
}
