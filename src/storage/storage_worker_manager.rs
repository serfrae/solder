use super::{Storable, StorageWorker};
use crate::database::DatabasePool;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{WorkerHandle, WorkerManager, WorkerManagerConfig};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use crossbeam::queue::SegQueue;
use log::info;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_postgres::NoTls;

pub struct StorageWorkerManager<T>
where
	T: Storable + 'static,
{
	#[allow(dead_code)]
	config: WorkerManagerConfig,
	pool: Arc<ThreadPool>,
	db_pool: DatabasePool,
	workers: Vec<WorkerHandle>,
	storage_rx: Arc<SegQueue<T>>,
}

impl<T> StorageWorkerManager<T>
where
	T: Storable + 'static,
{
	pub async fn new(
		config: WorkerManagerConfig,
		db_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
		storage_rx: Arc<SegQueue<T>>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			config,
			pool,
			db_pool,
			workers: Vec::with_capacity(worker_threads),
			storage_rx,
		}
	}

	pub async fn initialize(&mut self) {
		info!("Initialiazing storage workers");
		for i in 0..self.workers.capacity() {
			info!("Initiailizing storage worker: {}", i);
			self.spawn_worker().await;
		}
	}

	pub async fn run(&mut self) -> Result<()> {
		self.initialize().await;
		let ctrl_c = tokio::spawn(async {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to listen for Ctrl+C");
			info!("Received Ctrl+C signal. Initiating shutdown...");
		});

		let _ = tokio::try_join!(ctrl_c);

		self.shutdown_all().await?;

		Ok(())
	}
}

impl<T> WorkerManager for StorageWorkerManager<T>
where
	T: Storable + 'static,
{
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
		Box::pin(async move {
			let worker = StorageWorker::new(
				self.storage_rx.clone(),
				Arc::clone(&self.pool),
				Arc::clone(&self.db_pool),
			);

			self.workers.push(worker);
		})
	}

	fn shutdown_worker(
		&mut self,
		handle: WorkerHandle,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
		Box::pin(async move {
			handle.shutdown().await?;
			Ok(())
		})
	}

	fn shutdown_all(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
		Box::pin(async move {
			let mut shutdown_tasks = Vec::new();

			for handle in self.workers.drain(..) {
				shutdown_tasks.push(handle.shutdown());
			}

			for task in shutdown_tasks {
				task.await?
			}

			Ok(())
		})
	}
}
