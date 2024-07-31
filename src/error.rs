use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("No decoded transaction")]
	NoDecodedTransaction,

	#[error("No transaction signature")]
	NoTxid,

	#[error("Block not found")]
	BlockNotFound,

	#[error("Could not process block data")]
	BlockProcessingError,

	#[error("No data")]
	NoData,

	#[error("Send channel error")]
	SendChannelError,

	#[error("Solana client error: {0}")]
	SolanaClientError(#[from] solana_client::client_error::ClientError),

	#[error("Database error: {0}")]
	DatabaseError(#[from] tokio_postgres::Error),

	#[error("Database connection pool error: {0}")]
	DbPoolError(#[from] bb8::RunError<tokio_postgres::Error>),

	#[error("API error: {0}")]
	AxumError(#[from] axum::Error),

	#[error("Pubsub client error: {0}")]
	PubsubClientError(#[from] solana_client::pubsub_client::PubsubClientError),

	#[error("JSON serialization error: {0}")]
	SerdeError(#[from] serde_json::Error),

	#[error("Worker error: {0}")]
	WorkerError(String),

	#[error("Signature parse error: {0}")]
	SignatureParseError(#[from] solana_sdk::signature::ParseSignatureError),

	#[error("Pubkey parse error: {0}")]
	PubkeyParseError(#[from] solana_sdk::pubkey::ParsePubkeyError),

	#[error("Hash parse error: {0}")]
	HashParseError(#[from] solana_sdk::hash::ParseHashError),

	#[error("Channel send error")]
	ChannelSendError,

	#[error("Unknown error: {0}")]
	Unknown(String),

	#[error("Join error: {0}")]
	JoinError(#[from] tokio::task::JoinError),

	#[error("Parse error: {0}")]
	ParseError(#[from] std::num::ParseIntError),

	#[error("Parse token type")]
	ParseTokenType,

    #[error("No token balances")]
    EmptyTokenBalances,

	#[error("Couldn't open config: {0}")]
	OpenFileError(#[from] std::io::Error),

	#[error("Config deserialization error: {0}")]
	ConfigDeserializationError(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
