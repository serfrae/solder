use crossbeam::queue::SegQueue;
use log::info;
use solder::{
	config::load_config, database::DatabaseManager, error::Result, websocket::client::Client,
	worker::ProcessingWorkerManager,
};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
	pretty_env_logger::init();

	let config = load_config("Config.toml")?;
	info!("Read config");
	let proc_config = config.processor.to();
	let queue = Arc::new(SegQueue::new());
	let (storage_tx, storage_rx) = mpsc::channel(100);

	info!("Creating solana client");
	let sol_ps_client = Client::new(config.client, queue.clone());

	info!("Creating proc_wm");
	let mut proc_wm = ProcessingWorkerManager::new(proc_config, queue, storage_tx, 5);

	info!("Creating dbm");
	let mut dbm = DatabaseManager::new(config.database, storage_rx).await?;

    info!("Starting subscription");
	let client_handle = tokio::spawn(async move { sol_ps_client.subscribe_logs().await });

    info!("Starting processing manager");
	let proc_wm_handle = tokio::spawn(async move { proc_wm.run().await });

    info!("Starting database manager");
	let db_handle = tokio::spawn(async move { dbm.run_writer().await });

    tokio::try_join!(client_handle, proc_wm_handle, db_handle)?;

	Ok(())
}
