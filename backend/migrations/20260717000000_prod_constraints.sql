-- Production safety constraints for trade ingestion.

CREATE UNIQUE INDEX IF NOT EXISTS trades_tx_hash_unique
  ON trades (tx_hash);

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'trades_amount_sol_nonnegative'
  ) THEN
    ALTER TABLE trades
      ADD CONSTRAINT trades_amount_sol_nonnegative CHECK (amount_sol >= 0);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'trades_fee_sol_nonnegative'
  ) THEN
    ALTER TABLE trades
      ADD CONSTRAINT trades_fee_sol_nonnegative CHECK (fee_sol >= 0);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'trades_slot_nonnegative'
  ) THEN
    ALTER TABLE trades
      ADD CONSTRAINT trades_slot_nonnegative CHECK (slot IS NULL OR slot >= 0);
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'trades_required_text'
  ) THEN
    ALTER TABLE trades
      ADD CONSTRAINT trades_required_text CHECK (
        length(btrim(vault_address)) > 0
        AND length(btrim(tx_hash)) > 0
        AND length(btrim(trade_type)) > 0
        AND length(btrim(status)) > 0
      );
  END IF;
END $$;
