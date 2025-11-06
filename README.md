# Ephemeral Vault Program (Solana / Anchor)

A secure smart-contract system enabling *gasless trading* on a dark-pool perpetual futures DEX.

This protocol introduces **ephemeral session-wallets** controlled by a parent wallet.  
Users can delegate temporary trading authority to a vault-wallet (PDA), enabling:

✅ Gasless trading (SOL auto-top-ups)  
✅ Controlled delegation (limit-based session access)  
✅ Automatic session cleanup + fund return  

---

## 🚀 Flow Overview
User Wallet → Create Ephemeral Vault → Approve Delegate (session wallet) → Auto-Deposit SOL → Execute Trades → Revoke Access / Cleanup


---

## 📦 Features

Feature | Description 
-----------------------
✅ Ephemeral Vault Creation | Creates a PDA vault per user 
✅ Delegation | User delegates trading authority to an ephemeral wallet 
✅ Auto-Deposit | Automatically deposits SOL for gas fees into the vault 
✅ Execute Trade | Trades on behalf of the user using delegated authority 
✅ Revoke Access | User can terminate the session at any time 
✅ Cleanup Vault | Anyone can cleanup expired sessions (reward included) 

---

## 🧠 Program Accounts

### `EphemeralVault`
Stores vault state:

Field | Type | Description 
--------------------------
`user_wallet` | `Pubkey` | Owner wallet 
`vault_pda` | `Pubkey` | PDA vault account 
`approved_amount` | `u64` | Max delegated balance 
`used_amount` | `u64` | Amount used 
`available_amount` | `u64` | Remaining amount 
`delegate_wallet` | `Option<Pubkey>` | Temporary session wallet 
`created_at` | `i64` | Timestamp 
`is_active` | `bool` | Session active flag 
`bump` | `u8` | PDA bump 

---

## 📜 Instructions

Instruction | Description 
--------------------------
`create_ephemeral_vault` | initializes a vault for a user 
`approve_delegate` | approves a session wallet 
`auto_deposit_for_trade` | automates SOL transfer for transaction fees 
`execute_trade` | performs the trade using vault funds 
`revoke_access` | terminates delegation session 
`cleanup_vault` | closes expired vaults + cleanup reward 

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

## 📄 License

MIT License © 2025 — Ephemeral Vault Protocol
Contributions are welcome — PRs, issues, and feature requests encouraged.