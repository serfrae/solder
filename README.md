# Solder - Solana Data Aggregator

## Setup
### Requirements
1. Postgres
2. Rust
3. Solana RPC (Helius, Triton, etc.)

### Setup Database
To setup the database table use this command in psql:
```
\i /PATH/TO/PROJECT/ROOT/schema.sql
```

It is also recommended to provide indexes for performance else searching by account or
transaction may take some time:
```
CREATE INDEX idx_transaction_accounts_signature ON transaction_accounts(signature);
CREATE INDEX idx_transaction_accounts_account ON transaction_accounts(account);
CREATE INDEX idx_transaction_accounts_block_time ON transaction_accounts(block_time);
```


### Configuration
This application uses worker pools, to configure the number of workers per task
please edit the `ConfigTemplate.toml` template provided and rename it to `Config.toml`. 
My personal tests require five (5) rpc workers to retrieve block data via api 
calls since the `block_subscribe` method is not available on unpaid plans. 
Processing and storage workers will vary based on your hardware specs, 
I typically configure the same number of workers for each task.

All database details in the template must be provided, including username and password.

## Calling APIs
The api endpoints are:
```
/api/block/{blockhash}
/api/slot/{slot_number}
/api/transaction/{signature}
/api/account/{pubkey}?from={YYYY-MM-DD}&to={YYYY-MM-DD}
```

For accounts and transactions, `from` and `to` are optional.

When calling `/api/accounts` omitting `from` and `to` will retrieve all transactions
made by that account, omitting only `from` will retrieve all transactions for 
that account up till `to`, conversly omitting only `to` will retrieve all transactions 
since `from` till the current date.

Results are not paginated.
