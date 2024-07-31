use super::Storable;
use crate::database::DatabasePool;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle};
use crossbeam::queue::SegQueue;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct StorageWorker<T>
where
	T: Storable,
{
	storage_queue: Arc<SegQueue<T>>,
	db_pool: DatabasePool,
}

impl<T> StorageWorker<T>
where
	T: Storable + Send + 'static,
{
	pub fn new(
		storage_queue: Arc<SegQueue<T>>,
		thread_pool: Arc<ThreadPool>,
		db_pool: DatabasePool,
	) -> WorkerHandle {
		WorkerHandle::new(
			Self {
				storage_queue,
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
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		let storage_queue = self.storage_queue.clone();
		Box::pin(async move {
			loop {
				match storage_queue.pop() {
					Some(data) => {
                        log::info!("[STORAGE] Received data");
						log::info!("[STORAGE] Queue length: {}", storage_queue.len());
						data.store(self.db_pool.clone())?.await?;
					}
					None => tokio::task::yield_now().await,
				}
			}
		})
	}
}
