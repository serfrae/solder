use super::Gettable;
use crate::config::ClientConfig;
use crate::error::Result;
use crate::pool::ThreadPool;
use crate::worker::{Worker, WorkerHandle};
use crossbeam::queue::SegQueue;
use log::info;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct RpcClientWorker<T: Gettable>
where
	T::Output: Send,
{
	pub config: ClientConfig,
	pub queue_in: Arc<SegQueue<T>>,
	pub processing_queue: Arc<SegQueue<T::Output>>,
}

impl<T: Gettable> RpcClientWorker<T>
where
	T::Output: Send + 'static,
{
	pub fn new(
		config: ClientConfig,
		queue_in: Arc<SegQueue<T>>,
		processing_queue: Arc<SegQueue<T::Output>>,
		thread_pool: Arc<ThreadPool>,
	) -> WorkerHandle {
		let url = config.get_url();
		info!("Url: {}", url);

		WorkerHandle::new(
			Self {
				config,
				queue_in,
				processing_queue,
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
				match self.queue_in.pop() {
					Some(data) => {
						info!("Getting data");
						info!("WS -> RPC queue length: {}", self.queue_in.len());
						let output = T::get(data, &self.config).await?;
						info!("Pushing to processor");
						self.processing_queue.push(output);
					}
					None => tokio::task::yield_now().await,
				}
			}
		})
	}
}
