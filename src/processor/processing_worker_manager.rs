use super::{Processable, ProcessingWorker};
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::storage::Storable;
use crate::worker::{WorkerHandle, WorkerManager, WorkerManagerConfig};
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::info;
use std::sync::Arc;

pub struct ProcessingWorkerManager<T, U>
where
	T: Processable,
	U: From<T::ProcessedOutput> + Storable,
{
	config: WorkerManagerConfig,
	pool: Arc<ThreadPool>,
	workers: Vec<WorkerHandle>,
	processing_queue: Arc<SegQueue<T>>,
	storage_queue: Arc<SegQueue<U>>,
}

impl<T, U> ProcessingWorkerManager<T, U>
where
	T: Processable,
	U: From<T::ProcessedOutput> + Storable,
{
	pub fn new(
		config: WorkerManagerConfig,
		processing_queue: Arc<SegQueue<T>>,
		storage_queue: Arc<SegQueue<U>>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			config,
			pool,
			workers: Vec::with_capacity(worker_threads),
			processing_queue,
			storage_queue,
		}
	}

	pub async fn initialize(&mut self) {
		info!("Initialiazing minimum workers");
		for i in 0..self.workers.capacity() {
			info!("Initiailizing worker: {}", i);
			self.spawn_worker().await;
			info!("Worker initialized");
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

#[async_trait]
impl<T, U> WorkerManager for ProcessingWorkerManager<T, U>
where
	T: Processable,
	U: From<T::ProcessedOutput> + Storable,
{
	async fn spawn_worker(&mut self) {
		let worker = ProcessingWorker::new(
			self.processing_queue.clone(),
			self.storage_queue.clone(),
			Arc::clone(&self.pool),
		);

		self.workers.push(worker);
	}

	async fn shutdown_worker(&mut self, handle: WorkerHandle) -> Result<()> {
		handle.shutdown().await?;
		Ok(())
	}

	async fn shutdown_all(mut self) -> Result<()> {
		let mut shutdown_tasks = Vec::new();

		for handle in self.workers.drain(..) {
			shutdown_tasks.push(handle.shutdown());
		}

		for task in shutdown_tasks {
			task.await?
		}

		Ok(())
	}
}
