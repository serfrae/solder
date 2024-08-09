use crate::error::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub client: ClientConfig,
	pub processor: WorkerConfig,
	pub storage: WorkerConfig,
	pub database: DatabaseConfig,
	pub server: ServerConfig,
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
pub struct WorkerConfig {
	pub worker_threads: u32,
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
	pub port: u16,
}


pub fn load_config(file_path: &str) -> Result<Config> {
	let mut file = File::open(file_path)?;
	let mut contents = String::new();

	let _ = file.read_to_string(&mut contents)?;

	let config: Config = toml::de::from_str(&contents)?;

	Ok(config)
}
