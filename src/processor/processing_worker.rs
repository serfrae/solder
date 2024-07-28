use super::Processable;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::storage::Storable;
use crate::worker::{Worker, WorkerHandle};
use crossbeam::queue::SegQueue;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct ProcessingWorker<T, U>
where
	T: Processable,
	U: From<T::ProcessedOutput> + Storable,
{
	processing_queue: Arc<SegQueue<T>>,
	storage_queue: Arc<SegQueue<U>>,
}

impl<T, U> ProcessingWorker<T, U>
where
	T: Processable + 'static,
	U: From<T::ProcessedOutput> + Storable + 'static,
{
	pub fn new(
		processing_queue: Arc<SegQueue<T>>,
		storage_queue: Arc<SegQueue<U>>,
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

impl<T, U> Worker for ProcessingWorker<T, U>
where
	T: Processable + 'static,
	U: From<T::ProcessedOutput> + Storable + 'static,
{
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		let processing_queue = self.processing_queue.clone();
		let storage_queue = self.storage_queue.clone();

		Box::pin(async move {
			loop {
				match processing_queue.pop() {
					Some(data) => {
						let processed = data.process()?;
						let output = U::from(processed);
						storage_queue.push(output);
					}
					None => tokio::task::yield_now().await,
				}
			}
		})
	}
}
