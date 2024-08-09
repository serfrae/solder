use super::Storable;
use crate::database::DatabasePool;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle, WorkerManager};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_postgres::NoTls;

/// Manages the pool of `StorageWorker`s crossbeam channel is cloned to every
/// worker so that they can pull the next block of transactions when they are done with their task.
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

	/// Initialise the storage workers
	pub async fn initialize(&mut self) {
		log::info!("Initialiazing storage workers");
		for i in 0..self.workers.capacity() {
			log::info!("Initiailizing storage worker: {}", i);
			self.spawn_worker().await;
		}
	}

    /// Initialises all workers
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
    /// Spawns storage workers and stores their handles in `self.workers`
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

    /// Shutdown a worker using its handle
	fn shutdown_worker(
		&mut self,
		handle: WorkerHandle,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
		Box::pin(async move {
			handle.shutdown().await?;
			Ok(())
		})
	}

    /// Shutdown all workers by iterating through their join handles
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

/// Receives messages on a crossbeam channel, crossbeam channels are meant to be thread safe
/// and should not require locking. Each worker will pull a task off the channel as soon as it
/// arrives provided there are idle workers/enough workers in the pool. If enough workers aren't
/// defined or database writes are slow, this can cause a memory leak as the crossbeam channel
/// is unbounded.
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
    /// Runs the receiver loop storing data whenever it is received from the channel
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
