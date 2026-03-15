# Ephemeral-Vault Backend

Axum API that:

- Reads the on-chain `EphemeralVault` account (Solana devnet by default)
- Builds **unsigned** Anchor-compatible transactions for vault actions
- Stores trade history in Postgres (table exists; ingestion/indexing is separate)

## Local Run

1. Start Postgres:

```sh
cd /Users/averek7/Projects/Ephemeral-Vault/backend
docker compose up -d
```

2. Configure env:

- Create `backend/.env` using `.env.example` as a starting point.
- Ensure `RPC_URL` and `PROGRAM_ID` match the cluster where the Anchor program is deployed.
  - If you're running Anchor `localnet`, use `RPC_URL=http://127.0.0.1:8899`.

3. Run server:

```sh
cd /Users/averek7/Projects/Ephemeral-Vault/backend
cargo run
```

The server runs migrations from `backend/migrations/` on startup.

## Endpoints

- `GET /health`
- `GET /vault/:user_pubkey`
- `GET /vault_stats/:user_pubkey`
- `GET /trades/:vault_pubkey?limit=&offset=`
- `POST /trades` inserts a trade record into Postgres (optional; useful for bots/indexers)
- `POST /tx/*` returns `{ transactionBase64, vaultPda }` for the frontend wallet to sign and send.
