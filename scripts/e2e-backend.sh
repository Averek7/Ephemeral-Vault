#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${TMPDIR:-/tmp}/ev-e2e"
mkdir -p "$LOG_DIR"

VALIDATOR_LOG="$LOG_DIR/validator.log"
BACKEND_LOG="$LOG_DIR/backend.log"

cleanup() {
  if [[ -n "${BACKEND_PID:-}" ]] && kill -0 "$BACKEND_PID" >/dev/null 2>&1; then
    kill "$BACKEND_PID" || true
  fi
  if [[ -n "${VALIDATOR_PID:-}" ]] && kill -0 "$VALIDATOR_PID" >/dev/null 2>&1; then
    kill "$VALIDATOR_PID" || true
  fi
  (cd "$ROOT/backend" && docker compose down) >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "Starting Solana test validator..."
solana-test-validator --reset --quiet --ledger "$LOG_DIR/ledger" >"$VALIDATOR_LOG" 2>&1 &
VALIDATOR_PID=$!

echo "Waiting for localnet RPC..."
for _ in $(seq 1 30); do
  if curl -s http://127.0.0.1:8899 >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

solana config set --url http://127.0.0.1:8899 >/dev/null

ANCHOR_WALLET="${LOG_DIR}/anchor-wallet.json"
if [[ ! -f "$ANCHOR_WALLET" ]]; then
  solana-keygen new --no-bip39-passphrase -o "$ANCHOR_WALLET" -s >/dev/null
fi
export ANCHOR_WALLET
export ANCHOR_PROVIDER_URL="http://127.0.0.1:8899"
solana config set --keypair "$ANCHOR_WALLET" >/dev/null

DEPLOY_PUBKEY="$(solana-keygen pubkey "$ANCHOR_WALLET")"
solana airdrop 5 "$DEPLOY_PUBKEY" >/dev/null

echo "Building and deploying program..."
(cd "$ROOT/programs/ephemeralvault" && cargo +nightly build-sbf -- --locked >/dev/null)
PROGRAM_SO="$ROOT/target/deploy/ephemeralvault.so"
solana program deploy "$PROGRAM_SO" >/dev/null
PROGRAM_ID="$(solana address -k "$ROOT/target/deploy/ephemeralvault-keypair.json")"

echo "Starting Postgres..."
(cd "$ROOT/backend" && docker compose up -d) >/dev/null

echo "Starting backend..."
export RPC_URL="http://127.0.0.1:8899"
export PROGRAM_ID
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/ephemeral_vault"
export SERVER_HOST="127.0.0.1"
export SERVER_PORT="8080"
(cd "$ROOT/backend" && cargo run >"$BACKEND_LOG" 2>&1) &
BACKEND_PID=$!

echo "Waiting for backend /health..."
for _ in $(seq 1 40); do
  if curl -s http://127.0.0.1:8080/health >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "Running E2E backend tx-builder flow..."
BACKEND_URL="http://127.0.0.1:8080" RPC_URL="http://127.0.0.1:8899" node "$ROOT/scripts/e2e-backend.js"

echo "Done."
