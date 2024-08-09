use crate::error::Result;
use std::future::Future;
use std::sync::Arc;

/// Thread pool for workers
pub struct ThreadPool {
	runtime: Arc<tokio::runtime::Runtime>,
}

impl ThreadPool {
    /// Creates a worker pool whose size is the value of `worker_threads`
	pub fn new(worker_threads: usize) -> Self {
		let runtime = Arc::new(
			tokio::runtime::Builder::new_multi_thread()
				.worker_threads(worker_threads)
				.enable_all()
				.build()
				.expect("Failed to create Tokio runtime"),
		);

		Self { runtime }
	}

    /// Get join handles on execution
	pub fn execute<F>(&self, f: F) -> tokio::task::JoinHandle<Result<()>>
	where
		F: Future<Output = Result<()>> + Send + 'static,
	{
		let runtime = Arc::clone(&self.runtime);

		runtime.spawn(async move { f.await })
	}
}
