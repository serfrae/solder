use super::Processable;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle};
use crossbeam::queue::SegQueue;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct ProcessingWorker<T: Processable>
where
	T::Output: Send,
{
	processing_queue: Arc<SegQueue<T>>,
	storage_queue: Arc<SegQueue<T::Output>>,
}

impl<T: Processable> ProcessingWorker<T>
where
	T::Output: Send + 'static,
{
	pub fn new(
		processing_queue: Arc<SegQueue<T>>,
		storage_queue: Arc<SegQueue<T::Output>>,
		thread_pool: Arc<ThreadPool>,
	) -> WorkerHandle {
		WorkerHandle::new(
			Self {
				processing_queue,
				storage_queue,
			},
			thread_pool,
		)
	}
}

impl<T: Processable> Worker for ProcessingWorker<T>
where
	T::Output: Send + 'static,
{
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		let processing_queue = self.processing_queue.clone();
		let storage_queue = self.storage_queue.clone();

		Box::pin(async move {
			loop {
				match processing_queue.pop() {
					Some(data) => {
						log::info!("[PROCESSING] Received data");
                        log::info!("[PROCESSING] Queue length: {}", processing_queue.len());
						let processed = data.process()?;
						log::info!("[PROCESSING] Done");
						storage_queue.push(processed);
                        log::info!("[PROCESSING] Storage queue length: {}", storage_queue.len());
					}
					None => tokio::task::yield_now().await,
				}
			}
		})
	}
}
