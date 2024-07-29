use crossbeam::queue::SegQueue;
use log::info;
use solana_transaction_status::EncodedConfirmedBlock;
use solana_client::{rpc_response::SlotInfo, pubsub_client::SlotsSubscription};
use solder::{
	config::load_config,
	database::create_database_pool,
	error::Result,
	models::ProcessedBlockAndTransactions,
	processor::ProcessingWorkerManager,
	storage::StorageWorkerManager,
	client::ws::WsClient,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let config = load_config("Config.toml")?;
	info!("Read config");
	let proc_config = config.processor.to();
	let processing_queue = Arc::new(SegQueue::<EncodedConfirmedBlock>::new());
	let storage_queue = Arc::new(SegQueue::<ProcessedBlockAndTransactions>::new());
    let slot_queue = Arc::new(SegQueue::<SlotInfo>::new());

	let sol_ws_client = WsClient::<SlotsSubscription>::new(config.client.clone(), slot_queue.clone());

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
	let _ws_handle = tokio::spawn(async move { sol_ws_client.subscribe().await });

	info!("Starting processing manager");
	let _proc_handle = tokio::spawn(async move { proc_wm.run().await });

	info!("Starting database manager");
	let _db_handle = tokio::spawn(async move { storage_wm.await.run().await });

    //tokio::try_join!(ws_handle, rpc_handle, proc_handle, db_handle);

	tokio::signal::ctrl_c().await?;

	Ok(())
}
