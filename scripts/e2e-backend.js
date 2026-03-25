/* eslint-disable no-console */
const {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
} = require("@solana/web3.js");

const RPC_URL = process.env.RPC_URL || "http://127.0.0.1:8899";
const BACKEND_URL = process.env.BACKEND_URL || "http://127.0.0.1:8080";

const fetchFn = global.fetch;
if (!fetchFn) {
  throw new Error("Global fetch is not available. Use Node 18+.");
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function airdrop(connection, pubkey, sol) {
  const sig = await connection.requestAirdrop(
    pubkey,
    Math.round(sol * LAMPORTS_PER_SOL),
  );
  await connection.confirmTransaction(sig, "confirmed");
}

async function apiGet(path) {
  const res = await fetchFn(`${BACKEND_URL}${path}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GET ${path} failed: ${res.status} ${text}`);
  }
  return res.json();
}

async function apiPost(path, body) {
  const res = await fetchFn(`${BACKEND_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`POST ${path} failed: ${res.status} ${text}`);
  }
  return res.json();
}

async function sendBase64Tx(connection, base64, signer) {
  const tx = Transaction.from(Buffer.from(base64, "base64"));
  tx.partialSign(signer);
  const sig = await connection.sendRawTransaction(tx.serialize());
  await connection.confirmTransaction(sig, "confirmed");
  return sig;
}

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");

  const user = Keypair.generate();
  const delegate = Keypair.generate();

  console.log("Airdropping funds...");
  await airdrop(connection, user.publicKey, 2);
  await airdrop(connection, delegate.publicKey, 1);

  const approvedAmount = 1 * LAMPORTS_PER_SOL;
  const initialDeposit = 0.02 * LAMPORTS_PER_SOL;
  const tradeFee = 0.001 * LAMPORTS_PER_SOL;
  const tradeAmount = 0.01 * LAMPORTS_PER_SOL;

  console.log("Creating vault via backend...");
  const createResp = await apiPost("/tx/create_vault", {
    userPubkey: user.publicKey.toBase58(),
    approvedAmountLamports: Math.round(approvedAmount),
    delegatePubkey: delegate.publicKey.toBase58(),
    customDurationSeconds: 600,
    initialDepositLamports: Math.round(initialDeposit),
  });
  await sendBase64Tx(connection, createResp.transactionBase64, user);

  console.log("Fetching vault...");
  const vault = await apiGet(`/vault/${user.publicKey.toBase58()}`);
  assert(vault.status === "active", "vault should be active");
  assert(
    vault.delegate === delegate.publicKey.toBase58(),
    "delegate should match",
  );

  console.log("Executing trade via backend (delegate signed)...");
  const execResp = await apiPost("/tx/execute_trade", {
    vaultPubkey: vault.address,
    delegatePubkey: delegate.publicKey.toBase58(),
    tradeFeeLamports: Math.round(tradeFee),
    tradeAmountLamports: Math.round(tradeAmount),
  });
  await sendBase64Tx(connection, execResp.transactionBase64, delegate);

  const statsAfterTrade = await apiGet(`/vault_stats/${user.publicKey.toBase58()}`);
  assert(statsAfterTrade.tradeCount >= 1, "trade_count should be >= 1");

  console.log("Pausing vault...");
  const pauseResp = await apiPost("/tx/pause", {
    userPubkey: user.publicKey.toBase58(),
  });
  await sendBase64Tx(connection, pauseResp.transactionBase64, user);
  const paused = await apiGet(`/vault/${user.publicKey.toBase58()}`);
  assert(paused.status === "paused", "vault should be paused");

  console.log("Unpausing vault...");
  const unpauseResp = await apiPost("/tx/unpause", {
    userPubkey: user.publicKey.toBase58(),
  });
  await sendBase64Tx(connection, unpauseResp.transactionBase64, user);
  const unpaused = await apiGet(`/vault/${user.publicKey.toBase58()}`);
  assert(unpaused.status === "active", "vault should be active");

  console.log("Revoking access...");
  const revokeResp = await apiPost("/tx/revoke", {
    userPubkey: user.publicKey.toBase58(),
  });
  await sendBase64Tx(connection, revokeResp.transactionBase64, user);
  const revoked = await apiGet(`/vault/${user.publicKey.toBase58()}`);
  assert(revoked.status === "revoked", "vault should be revoked");

  console.log("Reactivating vault...");
  const reactivateResp = await apiPost("/tx/reactivate", {
    userPubkey: user.publicKey.toBase58(),
  });
  await sendBase64Tx(connection, reactivateResp.transactionBase64, user);
  const reactivated = await apiGet(`/vault/${user.publicKey.toBase58()}`);
  assert(reactivated.status === "active", "vault should be active after reactivation");

  console.log("E2E backend tx builder flow: OK");
  await sleep(250);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

