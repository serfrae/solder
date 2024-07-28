use super::WorkerHandle;
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

pub enum WorkerMode {
	Pool,
	Scale,
}

#[derive(Debug, Deserialize)]
pub struct WorkerManagerConfig {
	pub mode: WorkerMode,
	pub scale_config: Option<WorkerScalingConfig>,
}

pub struct WorkerScalingConfig {
	pub scale_up_threshold: usize,
	pub scale_down_threshold: usize,
	pub min_workers: usize,
	pub max_workers: usize,
	pub interval: Duration,
}

#[async_trait]
pub trait WorkerManager {
	async fn spawn_worker(&mut self);

	async fn shutdown_worker(&mut self, handle: WorkerHandle) -> Result<()>;

	async fn shutdown_all(self) -> Result<()>;
}

#[async_trait]
pub trait WorkerMonitor {
	async fn monitor_and_scale(self);

	async fn scale_up(&self);

	async fn scale_down(&self);
}
