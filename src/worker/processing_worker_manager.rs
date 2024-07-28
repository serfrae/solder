use super::{ProcessingWorker, WorkerHandle, WorkerManager, WorkerManagerConfig};
use crate::error::Result;
use crate::models::{Logs, TransactionRaw};
use crate::pool::ThreadPool;
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use log::info;

pub struct ProcessingWorkerManager {
	config: WorkerManagerConfig,
	pool: Arc<ThreadPool>,
	semaphore: Arc<Semaphore>,
	workers: Vec<WorkerHandle>,
	processing_queue: Arc<SegQueue<Logs>>,
	storage_tx: mpsc::Sender<Vec<TransactionRaw>>,
}

impl ProcessingWorkerManager {
	pub fn new(
		config: WorkerManagerConfig,
		processing_queue: Arc<SegQueue<Logs>>,
		storage_tx: mpsc::Sender<Vec<TransactionRaw>>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));
		let semaphore = Arc::new(Semaphore::new(config.min_workers));

		Self {
			config,
			pool,
			semaphore,
			workers: Vec::new(),
			processing_queue,
			storage_tx,
		}
	}

	pub async fn initialize(&mut self) {
        info!("Initialiazing minimum workers");
		for i in 0..self.config.min_workers {
            info!("Initiailizing worker: {}", i); 
			self.spawn_worker().await;
            info!("Worker initialized");
		}
	}

	pub async fn run(&mut self) -> Result<()> {
		self.initialize().await;
		loop {}
	}
}

#[async_trait]
impl WorkerManager for ProcessingWorkerManager {
	async fn spawn_worker(&mut self) { 
		let _permit = self.semaphore.acquire().await;

		let worker = ProcessingWorker::new(
			self.processing_queue.clone(),
			self.storage_tx.clone(),
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
