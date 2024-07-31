use super::WorkerHandle;
use crate::error::Result;
use serde::Deserialize;
use std::time::Duration;
use std::pin::Pin;
use std::future::Future;

#[derive(Clone, Debug, Deserialize)]
pub enum WorkerMode {
	Pool,
	Scale,
}

impl WorkerMode {
	pub fn from_str(s: &str) -> Self {
		match s {
			"pool" => Self::Pool,
			"scale" => Self::Scale,
			_ => Self::Pool,
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerManagerConfig {
	pub mode: WorkerMode,
	pub scale_config: Option<WorkerScalingConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkerScalingConfig {
	pub scale_up_threshold: usize,
	pub scale_down_threshold: usize,
	pub min_workers: usize,
	pub max_workers: usize,
	pub interval: Duration,
}

pub trait WorkerManager {
	fn spawn_worker(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

	fn shutdown_worker(&mut self, handle: WorkerHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

	fn shutdown_all(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

pub trait WorkerMonitor {
	fn monitor_and_scale(self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

	fn scale_up(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

	fn scale_down(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
