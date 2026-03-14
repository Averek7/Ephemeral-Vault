-- Core tables for backend persistence (trades + periodic vault snapshots).

CREATE TABLE IF NOT EXISTS trades (
  id uuid PRIMARY KEY,
  vault_address text NOT NULL,
  tx_hash text NOT NULL,
  trade_type text NOT NULL,
  amount_sol double precision NOT NULL,
  fee_sol double precision NOT NULL,
  status text NOT NULL,
  slot bigint NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_trades_vault_created_at
  ON trades (vault_address, created_at DESC);

CREATE TABLE IF NOT EXISTS vault_snapshots (
  id uuid PRIMARY KEY,
  vault_address text NOT NULL,
  owner text NOT NULL,
  balance_sol double precision NULL,
  approved_amount_sol double precision NULL,
  trades_executed integer NULL,
  status text NULL,
  snapshot_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vault_snapshots_vault_snapshot_at
  ON vault_snapshots (vault_address, snapshot_at DESC);
