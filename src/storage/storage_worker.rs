use super::Storable;
use crate::database::DatabasePool;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle, WorkerManager};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub struct StorageWorkerManager<T>
where
	T: Storable + 'static,
{
	pool: Arc<ThreadPool>,
	db_pool: DatabasePool,
	workers: Vec<WorkerHandle>,
	storage_rx: crossbeam_channel::Receiver<T>,
}

impl<T> StorageWorkerManager<T>
where
	T: Storable + 'static,
{
	pub async fn new(
		db_pool: Arc<Pool<PostgresConnectionManager<NoTls>>>,
		storage_rx: crossbeam_channel::Receiver<T>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			pool,
			db_pool,
			workers: Vec::with_capacity(worker_threads),
			storage_rx,
		}
	}

	pub async fn initialize(&mut self) {
		log::info!("Initialiazing storage workers");
		for i in 0..self.workers.capacity() {
			log::info!("Initiailizing storage worker: {}", i);
			self.spawn_worker().await;
		}
	}

	pub async fn run(&mut self) -> Result<()> {
		self.initialize().await;
		let ctrl_c = tokio::spawn(async {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to listen for Ctrl+C");
			log::info!("Received Ctrl+C signal. Initiating shutdown...");
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

pub struct StorageWorker<T>
where
	T: Storable,
{
	storage_rx: crossbeam_channel::Receiver<T>,
	db_pool: DatabasePool,
}

impl<T> StorageWorker<T>
where
	T: Storable + Send + 'static,
{
	pub fn new(
		storage_rx: crossbeam_channel::Receiver<T>,
		thread_pool: Arc<ThreadPool>,
		db_pool: DatabasePool,
	) -> WorkerHandle {
		WorkerHandle::new(
			Self {
				storage_rx,
				db_pool,
			},
			thread_pool,
		)
	}
}

impl<T> Worker for StorageWorker<T>
where
	T: Storable + Send + 'static,
{
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		Box::pin(async move {
			loop {
				match self.storage_rx.recv() {
					Ok(data) => {
						log::debug!("[STORAGE] Queue length: {}", self.storage_rx.len());
						match data.store(self.db_pool.clone())?.await {
							Ok(..) => continue,
							Err(e) => {
								log::error!("Database error: {}", e);
								continue;
							}
						}
					}
					Err(e) => {
						log::error!("[STORAGE] Error receiving data: {}", e);
						tokio::task::yield_now().await;
					}
				}
			}
		})
	}
}
