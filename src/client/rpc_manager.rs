use super::rpc_worker::RpcClientWorker;
use super::Gettable;
use crate::config::ClientConfig;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{WorkerHandle, WorkerManager};
use crossbeam::queue::SegQueue;
use std::future::Future;
use std::pin::Pin;
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

impl<T> WorkerManager for RpcWorkerManager<T>
where
	T: Gettable + 'static,
	T::Output: Send + 'static,
{
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
		Box::pin(async move {
			let worker = RpcClientWorker::new(
				self.config.clone(),
				self.queue_in.clone(),
				self.processing_queue.clone(),
				self.pool.clone(),
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
			let mut shutdown_tasks = Vec::with_capacity(self.workers.capacity());

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
