use super::{Worker, WorkerHandle};
use crate::error::{AppError, Result};
use crate::models::{Logs, TransactionRaw};
use crate::pool::ThreadPool;
use crate::processor::Processor;
use crossbeam::queue::SegQueue;
use log::info;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ProcessingWorker {
	processing_queue: Arc<SegQueue<Logs>>,
	storage_tx: mpsc::Sender<Vec<TransactionRaw>>,
}

impl ProcessingWorker {
	pub fn new(
		processing_queue: Arc<SegQueue<Logs>>,
		storage_tx: mpsc::Sender<Vec<TransactionRaw>>,
		thread_pool: Arc<ThreadPool>,
	) -> WorkerHandle {
		WorkerHandle::new(
			Self {
				processing_queue,
				storage_tx,
			},
			thread_pool,
		)
	}
}

impl Worker for ProcessingWorker {
	fn run(self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
		Box::pin(async move {
			loop {
				match self.processing_queue.pop() {
					Some(log) => {
						info!("Log received: {}", log.context.slot);
						let transaction = Processor::process_log(log)?;
						info!("Log processed");
						if transaction.is_empty() {
                            info!("Processed log was empty");
							continue;
						}
						self.storage_tx
							.send(transaction)
							.await
							.map_err(|_| AppError::SendChannelError)?;
					}
					None => continue,
				}
			}
		})
	}
}
