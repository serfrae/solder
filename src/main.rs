use crossbeam::queue::SegQueue;
use log::info;
use solana_client::{pubsub_client::SlotsSubscription, rpc_response::SlotInfo};
use solana_transaction_status::UiConfirmedBlock;
use solder::{
	client::rpc_manager::RpcWorkerManager, client::ws::WsClient, config::load_config,
	database::create_database_pool, error::Result, models::Aggregate,
	processor::ProcessingWorkerManager, storage::StorageWorkerManager,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let config = load_config("Config.toml")?;
	info!("Read config");
	let proc_config = config.processor.to();

	let storage_queue = Arc::new(SegQueue::<Vec<Option<Aggregate>>>::new());
	let rpc_queue_in = Arc::new(SegQueue::<SlotInfo>::new());
	let processing_queue = Arc::new(SegQueue::<(SlotInfo, UiConfirmedBlock)>::new());

	let sol_ws_client =
		WsClient::<SlotsSubscription>::new(config.client.clone(), rpc_queue_in.clone());

    info!("Creating rpc_wm");
	let mut rpc_wm = RpcWorkerManager::<SlotInfo>::new(
		config.client,
		rpc_queue_in,
		processing_queue.clone(),
		config.processor.worker_threads as usize,
	);
	info!("Creating db_pool");
	let db_pool = create_database_pool(&config.database).await?;

	info!("Creating proc_wm");
	let mut proc_wm = ProcessingWorkerManager::new(
		proc_config.clone(),
		processing_queue,
		storage_queue.clone(),
		config.processor.worker_threads as usize,
	);

	info!("Creating storage_wm");
	let storage_wm = StorageWorkerManager::new(proc_config, db_pool, storage_queue, 12);

	info!("Starting subscription");
	let _ws_handle = tokio::spawn(async move { sol_ws_client.subscribe().await });

	info!("Starting rpc_wm");
	let _rpc_handle = tokio::spawn(async move { rpc_wm.run().await });

	info!("Starting proc_wm");
	let _proc_handle = tokio::spawn(async move { proc_wm.run().await });

	info!("Starting storage_wm");
	let _db_handle = tokio::spawn(async move { storage_wm.await.run().await });

	//tokio::try_join!(ws_handle, rpc_handle, proc_handle, db_handle);

	tokio::signal::ctrl_c().await?;

	Ok(())
}
