use crate::error::Result;
use crate::worker::{WorkerManagerConfig, WorkerMode, WorkerScalingConfig};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub database: DatabaseConfig,
	pub server: ServerConfig,
	pub processor: ProcessorConfig,
	pub client: ClientConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClientConfig {
	pub url: String,
	pub api_key: String,
}

impl ClientConfig {
	pub fn get_ws_url(&self) -> String {
		if !self.api_key.is_empty() {
			format!("wss://{}/?api-key={}", self.url, self.api_key)
		} else {
			format!("wss://{}/", self.url)
		}
	}

	pub fn get_url(&self) -> String {
		if !self.api_key.is_empty() {
			format!("https://{}/?api-key={}", self.url, self.api_key)
		} else {
			format!("https://{}/", self.url)
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
	pub user: String,
	pub password: String,
	pub port: u16,
	pub host: String,
	pub pool_size: u32,
	pub db_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
	pub host: String,
	pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ProcessorConfig {
	pub worker_mode: String,
	pub worker_threads: u32,
	pub min_workers: u16,
	pub interval: u64,
	pub scale_up_threshold: u64,
	pub scale_down_threshold: u64,
}

impl ProcessorConfig {
	pub fn to(&self) -> WorkerManagerConfig {
		let mode = WorkerMode::from_str(self.worker_mode.as_str());
		WorkerManagerConfig {
			mode,
			scale_config: Some(WorkerScalingConfig {
				min_workers: self.min_workers as usize,
				max_workers: self.worker_threads as usize,
				interval: Duration::from_millis(self.interval),
				scale_up_threshold: self.scale_up_threshold as usize,
				scale_down_threshold: self.scale_down_threshold as usize,
			}),
		}
	}
}

pub fn load_config(file_path: &str) -> Result<Config> {
	let mut file = File::open(file_path)?;
	let mut contents = String::new();

	let _ = file.read_to_string(&mut contents)?;

	let config: Config = toml::de::from_str(&contents)?;

	Ok(config)
}
