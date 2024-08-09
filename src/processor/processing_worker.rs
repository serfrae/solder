use super::Processable;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle, WorkerManager};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Generic workers to process a `Processable` type. As crossbeam channels are unbounded,
/// enough workers should be in the worker pool or to retrieve tasks from the receiving channel.
/// A memory leak can occur if there are too few workers as the crossbeam channels are unbounded
pub struct ProcessingWorkerManager<T>
where
	T: Processable,
	T::Output: Send,
{
	pool: Arc<ThreadPool>,
	workers: Vec<WorkerHandle>,
	proc_rx: crossbeam_channel::Receiver<T>,
	storage_tx: crossbeam_channel::Sender<T::Output>,
}

impl<T> ProcessingWorkerManager<T>
where
	T: Processable,
	T::Output: Send + 'static,
{
	pub fn new(
		proc_rx: crossbeam_channel::Receiver<T>,
		storage_tx: crossbeam_channel::Sender<T::Output>,
		worker_threads: usize,
	) -> Self {
		let pool = Arc::new(ThreadPool::new(worker_threads));

		Self {
			pool,
			workers: Vec::with_capacity(worker_threads),
			proc_rx,
			storage_tx,
		}
	}

	pub async fn initialize(&mut self) {
		log::info!("Initialiazing minimum workers");
		for i in 0..self.workers.capacity() {
			log::info!("Initiailizing worker: {}", i);
			self.spawn_worker().await;
			log::info!("Worker initialized");
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

impl<T> WorkerManager for ProcessingWorkerManager<T>
where
	T: Processable + 'static,
	T::Output: Send + 'static,
{
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
		Box::pin(async move {
			let worker = ProcessingWorker::new(
				self.proc_rx.clone(),
				self.storage_tx.clone(),
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

pub struct ProcessingWorker<T: Processable>
where
	T::Output: Send,
{
	proc_rx: crossbeam_channel::Receiver<T>,
	storage_tx: crossbeam_channel::Sender<T::Output>,
}

impl<T: Processable> ProcessingWorker<T>
where
	T::Output: Send + 'static,
{
	pub fn new(
		proc_rx: crossbeam_channel::Receiver<T>,
		storage_tx: crossbeam_channel::Sender<T::Output>,
		thread_pool: Arc<ThreadPool>,
	) -> WorkerHandle {
		WorkerHandle::new(
			Self {
				proc_rx,
				storage_tx,
			},
			thread_pool,
		)
	}
}

/// Run loop for `ProcessorWorker`, if errors are thrown, log them an continue to next loop
impl<T: Processable> Worker for ProcessingWorker<T>
where
	T::Output: Send + 'static,
{
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		Box::pin(async move {
			loop {
				match self.proc_rx.recv() {
					Ok(data) => {
						log::debug!("[PROCESSING] Queue length: {}", self.proc_rx.len());
						let processed = match data.process() {
							Ok(data) => data,
							Err(e) => {
								log::error!("[PROCESSING] Could not process block: {}", e);
								tokio::task::yield_now().await;
								continue;
							}
						};
						match self.storage_tx.send(processed) {
							Ok(..) => continue,
							Err(e) => {
								log::error!("Error sending to storage worker: {}", e);
								tokio::task::yield_now().await;
							}
						}
					}
					Err(e) => {
						log::error!("Processing error: {}", e);
						tokio::task::yield_now().await;
					}
				}
			}
		})
	}
}
