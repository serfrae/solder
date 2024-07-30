use super::Gettable;
use super::rpc_worker::RpcClientWorker;
use crate::config::ClientConfig;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{WorkerHandle, WorkerManager};
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use std::sync::Arc;

pub struct RpcWorkerManager<T>
where
	T: Gettable,
	T::Output: Send,
{
	config: ClientConfig,
	pool: Arc<ThreadPool>,
	workers: Vec<WorkerHandle>,
	queue_in: Arc<SegQueue<T>>,
	processing_queue: Arc<SegQueue<T::Output>>,
}

impl<T> RpcWorkerManager<T>
where
	T: Gettable + 'static,
	T::Output: Send + 'static,
{
	pub fn new(
		config: ClientConfig,
		queue_in: Arc<SegQueue<T>>,
		processing_queue: Arc<SegQueue<T::Output>>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			config,
			pool,
			workers: Vec::with_capacity(worker_threads),
			queue_in,
			processing_queue,
		}
	}

	pub async fn initialize(&mut self) {
		log::info!("Initializing rpc workers...");
		for _ in 0..self.workers.capacity() {
			self.spawn_worker().await;
		}
	}

	pub async fn run(&mut self) -> Result<()> {
		self.initialize().await;
		let ctrl_c = tokio::spawn(async {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to listen for Ctrl+C");
			log::info!("Received Ctrl+C signal. Initiating shtudown...");
		});

		let _ = tokio::try_join!(ctrl_c);

		self.shutdown_all().await?;

		Ok(())
	}
}

#[async_trait]
impl<T> WorkerManager for RpcWorkerManager<T>
where
	T: Gettable + 'static,
	T::Output: Send + 'static,
{
	async fn spawn_worker(&mut self) {
		let worker = RpcClientWorker::new(
			self.config.clone(),
			self.queue_in.clone(),
			self.processing_queue.clone(),
			self.pool.clone(),
		);
		self.workers.push(worker);
	}

	async fn shutdown_worker(&mut self, handle: WorkerHandle) -> Result<()> {
		handle.shutdown().await?;
		Ok(())
	}

	async fn shutdown_all(&mut self) -> Result<()> {
		let mut shutdown_tasks = Vec::with_capacity(self.workers.capacity());

		for handle in self.workers.drain(..) {
			shutdown_tasks.push(handle.shutdown());
		}

		for task in shutdown_tasks {
			task.await?
		}

		Ok(())
	}
}
