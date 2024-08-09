use super::Gettable;
use crate::config::ClientConfig;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle, WorkerManager};
use log::info;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Manages the pool of `RpcWorkers`s. Crossbeam channel is cloned to every
/// worker to continuously retrieve blocks without creating a backlog. 
/// On a free plan with Helius, it takes longer than 400ms to retrieve one block, testing on my
/// connection/laptop requires five(5) rpc workers to ensure no backlog of requests. Backlogging
/// requests can lead to a memory leak as the crossbeam channels are unbounded.
pub struct RpcWorkerManager<T>
where
	T: Gettable,
	T::Output: Send,
{
	config: ClientConfig,
	pool: Arc<ThreadPool>,
	workers: Vec<WorkerHandle>,
	rpc_rx: crossbeam_channel::Receiver<T>,
	proc_tx: crossbeam_channel::Sender<T::Output>,
}

impl<T> RpcWorkerManager<T>
where
	T: Gettable + 'static,
	T::Output: Send + 'static,
{
	pub fn new(
		config: ClientConfig,
		rpc_rx: crossbeam_channel::Receiver<T>,
		proc_tx: crossbeam_channel::Sender<T::Output>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			config,
			pool,
			workers: Vec::with_capacity(worker_threads),
			rpc_rx,
			proc_tx,
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
				self.rpc_rx.clone(),
				self.proc_tx.clone(),
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

pub struct RpcClientWorker<T: Gettable>
where
	T::Output: Send,
{
	pub config: ClientConfig,
	pub rpc_rx: crossbeam_channel::Receiver<T>,
	pub proc_tx: crossbeam_channel::Sender<T::Output>,
}

impl<T: Gettable> RpcClientWorker<T>
where
	T::Output: Send + 'static,
{
	pub fn new(
		config: ClientConfig,
		rpc_rx: crossbeam_channel::Receiver<T>,
		proc_tx: crossbeam_channel::Sender<T::Output>,
		thread_pool: Arc<ThreadPool>,
	) -> WorkerHandle {
		let url = config.get_url();
		info!("Url: {}", url);

		WorkerHandle::new(
			Self {
				config,
				rpc_rx,
				proc_tx,
			},
			thread_pool,
		)
	}
}

impl<T: Gettable> Worker for RpcClientWorker<T>
where
	T::Output: Send + 'static,
{
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		Box::pin(async move {
			loop {
				match self.rpc_rx.recv() {
					Ok(data) => {
						log::debug!("WS -> RPC queue length: {}", self.rpc_rx.len());
						let output = match T::get(data, &self.config).await {
							Ok(output) => output,
							Err(e) => {
								log::error!("Error getting block: {}", e);
								continue;
							}
						};
						match self.proc_tx.send(output) {
							Ok(_) => continue,
							Err(e) => {
								log::error!("Error sending to processor: {}", e);
								continue;
							}
						}
					}
					Err(e) => {
						log::error!("{}", e);
						tokio::task::yield_now().await
					}
				}
			}
		})
	}
}
