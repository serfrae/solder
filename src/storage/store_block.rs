use super::Storable;
use crate::{error::Result, database::DatabasePool, models::ProcessedBlockAndTransactions};

impl Storable for ProcessedBlockAndTransactions {
	fn store(&self, db_pool: &DatabasePool) -> Result<()> {
		let transaction_query = "";
		let block_query = "";

		Ok(())
	}
}
