CREATE TABLE IF NOT EXISTS sessions (
  id uuid PRIMARY KEY,
  user_id text NOT NULL,
  ephemeral_pubkey text NOT NULL,
  encrypted_key bytea NOT NULL,
  vault_pda text NOT NULL,
  expires_at timestamptz NOT NULL,
  approved_amount bigint DEFAULT 0,
  total_deposited bigint DEFAULT 0,
  delegate_pubkey text,
  delegate_approved boolean DEFAULT false,
  created_at timestamptz DEFAULT now()
);
