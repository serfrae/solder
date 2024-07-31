use super::{Processable, ProcessingWorker};
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{WorkerHandle, WorkerManager, WorkerManagerConfig};
use crossbeam::queue::SegQueue;
use log::info;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct ProcessingWorkerManager<T>
where
	T: Processable,
	T::Output: Send,
{
	#[allow(dead_code)]
	config: WorkerManagerConfig,
	pool: Arc<ThreadPool>,
	workers: Vec<WorkerHandle>,
	processing_queue: Arc<SegQueue<T>>,
	storage_queue: Arc<SegQueue<T::Output>>,
}

impl<T> ProcessingWorkerManager<T>
where
	T: Processable,
	T::Output: Send + 'static,
{
	pub fn new(
		config: WorkerManagerConfig,
		processing_queue: Arc<SegQueue<T>>,
		storage_queue: Arc<SegQueue<T::Output>>,
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

impl<T> WorkerManager for ProcessingWorkerManager<T>
where
	T: Processable + 'static,
	T::Output: Send + 'static,
{
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
		Box::pin(async move {
			let worker = ProcessingWorker::new(
				self.processing_queue.clone(),
				self.storage_queue.clone(),
				Arc::clone(&self.pool),
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
