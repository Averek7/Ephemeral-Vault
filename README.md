# Ephemeral Vault Program (Solana / Anchor)

A secure smart-contract system enabling *gasless trading* on a dark‑pool perpetual futures DEX.

This protocol introduces **ephemeral session-wallets** controlled by a parent wallet.
Users can delegate temporary trading authority to a vault-wallet (PDA), enabling:

✅ Gasless trading (SOL auto-top-ups)  
✅ Controlled delegation (limit-based session access)  
✅ Automatic session cleanup + fund return  

---

## 🚀 Flow Overview

User Wallet → Create Ephemeral Vault → Approve Delegate (session wallet) → Auto‑Deposit SOL → Execute Trades → Revoke Access / Cleanup

---

## 📦 Features

Feature | Description
--------|------------
Ephemeral Vault Creation | Creates a PDA vault per user
Delegation | User delegates trading authority to an ephemeral wallet
Auto‑Deposit | Automatically deposits SOL for gas fees into the vault
Execute Trade | Performs the trade using vault funds
Revoke Access | User can terminate the session at any time
Cleanup Vault | Anyone can cleanup expired sessions (reward included)

---

## 🧠 Program Accounts

### EphemeralVault (Account)

Field | Type | Description
------|------|------------
user_wallet | Pubkey | Owner wallet
vault_pda | Pubkey | PDA vault account
approved_amount | u64 | Max delegated balance
used_amount | u64 | Amount used
available_amount | u64 | Remaining amount
delegate_wallet | Option<Pubkey> | Temporary session wallet
created_at | i64 | Timestamp
is_active | bool | Session active flag
bump | u8 | PDA bump

---

# 🧠 Ephemeral Wallet Backend (Rust / Solana)

A backend service managing **off‑chain ephemeral wallets** that interact with the Ephemeral Vault Solana program.

---

## 🏗️ Backend Architecture

Component | Responsibility
----------|--------------
Session Manager | Generates ephemeral wallets, encrypts private keys, tracks active sessions.
Delegation Manager | Builds and submits on-chain delegation instruction.
Auto‑Deposit Calculator | Estimates SOL gas fee requirements and triggers auto deposits.
Transaction Signer | Signs transactions in-memory using ephemeral wallet keys.
Vault Monitor | Cron job — auto closes expired sessions.
PostgreSQL Database | Stores metadata + audit logs (never private keys).

---

## 🔒 Security Principles

Principle | Description
----------|------------
No private key stored unencrypted | Only encrypted in memory with AES‑256.
Session wallets expire | Auto revoked after TTL.
No backend custody of funds | Ephemeral wallet operates within delegated bounds.
Least authority | Only trading authority granted.

---

## 🛠 Build & Deploy

### Install Dependencies
```sh
anchor --version
solana --version
rustup toolchain install stable
```

### Configure Solana, Build Program & Deploy
```sh
solana config set --url https://api.devnet.solana.com
anchor build
anchor deploy
```

## Test
![alt text](<Screenshot 2025-11-05 122523.png>)


# 🧩 Backend API (REST)

Base URL:  
`http://localhost:8080/`

---

### `POST /session/create`
Creates a new ephemeral session & wallet.

```json
{
  "user_wallet": "USER_MAIN_WALLET_PUBLIC_KEY"
}
```

---

### `POST /session/approve`
Approves delegation on-chain.

```json
{ "session_id": "uuid" }
```

---

### `POST /session/deposit`
Deposits SOL for trading fees.

```json
{
  "session_id": "uuid",
  "estimated_fee": 0.02
}
```

---

### `POST /session/execute-trade`

```json
{
  "session_id": "uuid",
  "market": "SOL/USDT",
  "side": "long",
  "size": "50 USDT"
}
```

---

### `DELETE /session/revoke`

```json
{ "session_id": "uuid" }
```

---

### `POST /session/cleanup`

```json
{ "session_id": "uuid" }
```

---

## 🗄 Database Schema (PostgreSQL)

Table: `ephemeral_sessions`

Field | Type | Description
------|------|------------
session_id | UUID | Session ID
user_wallet | TEXT | Parent wallet
ephemeral_wallet | TEXT | Disposable trading wallet
vault_pda | TEXT | PDA address
expires_at | TIMESTAMP | Expiry

---

## 🔄 Session Lifecycle

1. User requests session
2. Ephemeral wallet created
3. PDA vault initialized
4. Delegation approved
5. Auto‑deposit gas funds
6. Trade executed
7. Vault closed → key destroyed

---

## 🛠 Build & Deploy

```sh
anchor build
anchor deploy
```

Rust backend:

```sh
cargo run
```

Requires:

```
DATABASE_URL
RPC_URL
```

---

## 📄 License

MIT License © 2025 — Ephemeral Vault Protocol
