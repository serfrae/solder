use crossbeam::queue::SegQueue;
use log::info;
use solder::{
	config::load_config,
	database::create_database_pool,
	error::Result,
	models::{ProcessedBlockAndTransactions, RawBlock, ProcessedTransactionLogs, RawTransactionLogs},
	processor::ProcessingWorkerManager,
	storage::StorageWorkerManager,
	websocket::{client::Client, subscribe_blocks::BlockSubscription, subscribe_logs::TransactionLogsSubscription},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let config = load_config("Config.toml")?;
	info!("Read config");
	let proc_config = config.processor.to();
	let processing_queue = Arc::new(SegQueue::<RawTransactionLogs>::new());
	let storage_queue = Arc::new(SegQueue::<ProcessedTransactionLogs>::new());

	info!("Creating solana client");
	let sol_ps_client = Client::<TransactionLogsSubscription>::new(config.client, processing_queue.clone());

	info!("Creating db_pool");
	let db_pool = create_database_pool(&config.database).await?;

	info!("Creating proc_wm");
	let mut proc_wm = ProcessingWorkerManager::new(
		proc_config.clone(),
		processing_queue.clone(),
		storage_queue.clone(),
		5,
	);

	info!("Creating storage_wm");
	let storage_wm = StorageWorkerManager::new(proc_config, db_pool, storage_queue, 5);

	info!("Starting subscription");
	tokio::spawn(async move { sol_ps_client.subscribe().await });

	info!("Starting processing manager");
	tokio::spawn(async move { proc_wm.run().await });

	info!("Starting database manager");
	tokio::spawn(async move { storage_wm.await.run().await });

	//tokio::try_join!(client_handle, proc_wm_handle, db_handle)?;

	tokio::signal::ctrl_c().await?;

	Ok(())
}
