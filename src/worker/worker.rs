use crate::error::Result;
use crate::pool::ThreadPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Trait for worker managers, worker managers should have a vector field to hold `WorkerHandles`
/// to ensure that workers can be shutdown
pub trait WorkerManager {
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

	fn shutdown_worker(&mut self, handle: WorkerHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

	fn shutdown_all(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

pub trait Worker: Send + 'static {
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
}

/// Run is called in constructor to get join handle so toavoid uncessary fiddling with `Option` and `take()`
pub struct WorkerHandle {
	join_handle: JoinHandle<Result<()>>,
	shutdown_tx: mpsc::Sender<()>,
}

impl WorkerHandle {
	pub fn new<W: Worker>(worker: W, thread_pool: Arc<ThreadPool>) -> Self {
		let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

		let join_handle = thread_pool.execute(async move {
			let worker_future = worker.run();
			tokio::select! {
				result = worker_future => result,
				_ = shutdown_rx.recv() => Ok(()),
			}
		});

		Self {
			join_handle,
			shutdown_tx,
		}
	}

	pub async fn shutdown(self) -> Result<()> {
		if let Err(e) = self.shutdown_tx.send(()).await {
			log::error!("Failed to send shutdown signal: {}", e);
		}

		match self.join_handle.await {
			Ok(result) => result,
			Err(e) => {
				log::error!("Worked task failed: {}", e);
				Err(e.into())
			}
		}
	}
}
