CREATE TABLE transaction_accounts (
    blockhash TEXT NOT NULL,
    slot BIGINT NOT NULL,
    block_time BIGINT NOT NULL,
    signature TEXT NOT NULL,
    account TEXT NOT NULL,
    PRIMARY KEY (blockhash, signature, account)
);

CREATE INDEX idx_transaction_accounts_signature ON transaction_accounts(signature);
CREATE INDEX idx_transaction_accounts_account ON transaction_accounts(account);
CREATE INDEX idx_transaction_accounts_block_time ON transaction_accounts(block_time);
