use log::info;
use solana_client::{pubsub_client::SlotsSubscription, rpc_response::SlotInfo};
use solana_transaction_status::UiConfirmedBlock;
use solder::{
	api::server::Server, client::rpc_manager::RpcWorkerManager, client::ws::WsClient,
	config::load_config, database::create_database_pool, error::Result, models::Aggregate,
	processor::ProcessingWorkerManager, storage::StorageWorkerManager,
};

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let config = load_config("Config.toml")?;

	let (rpc_tx, rpc_rx) = crossbeam_channel::unbounded::<SlotInfo>();
	let (proc_tx, proc_rx) = crossbeam_channel::unbounded::<(SlotInfo, UiConfirmedBlock)>();
	let (storage_tx, storage_rx) = crossbeam_channel::unbounded::<Vec<Option<Aggregate>>>();

	let sol_ws_client = WsClient::<SlotsSubscription>::new(config.client.clone(), rpc_tx);

	info!("Creating rpc_wm");
	let mut rpc_wm = RpcWorkerManager::<SlotInfo>::new(
		config.client,
		rpc_rx,
		proc_tx,
		config.processor.worker_threads as usize,
	);

	info!("Creating db_pool");
	let db_pool = create_database_pool(&config.database).await?;

	info!("Creating proc_wm");
	let mut proc_wm = ProcessingWorkerManager::new(
		proc_config.clone(),
		proc_rx,
		storage_tx,
		config.processor.worker_threads as usize,
	);

	info!("Creating storage_wm");
	let storage_wm = StorageWorkerManager::new(db_pool.clone(), storage_rx, 12);

	info!("Creating server");
	let server = Server::new(db_pool.clone(), config.server.port);

	info!("Starting subscription");
	let _ws_handle = tokio::spawn(async move { sol_ws_client.subscribe().await });

	info!("Starting rpc_wm");
	let _rpc_handle = tokio::spawn(async move { rpc_wm.run().await });

	info!("Starting proc_wm");
	let _proc_handle = tokio::spawn(async move { proc_wm.run().await });

	info!("Starting storage_wm");
	let _db_handle = tokio::spawn(async move { storage_wm.await.run().await });

	info!("Running server");
	let _server_handle = tokio::spawn(async move { server.await.run().await });

	tokio::signal::ctrl_c().await?;

	Ok(())
}
