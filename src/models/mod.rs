pub mod transaction;

pub use transaction::{Transaction, TransactionRaw, TokenType};

pub type Logs = solana_client::rpc_response::Response<solana_client::rpc_response::RpcLogsResponse>;
