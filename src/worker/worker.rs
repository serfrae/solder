use crate::error::Result;
use crate::pool::ThreadPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub trait Worker: Send + 'static {
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
}

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
		let _ = self.shutdown_tx.send(()).await;
		self.join_handle.await?
	}
}
