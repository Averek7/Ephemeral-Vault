use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;


pub struct TransactionSigner {
    pub rpc: RpcClient,
}


impl TransactionSigner {
    pub fn new(rpc_url: &str) -> Self {
        Self { rpc: RpcClient::new(rpc_url.to_string()) }
    }


    pub async fn send_signed_transaction(&self, tx: Transaction) -> Result<solana_sdk::signature::Signature> {
        let sig = self.rpc.send_and_confirm_transaction(&tx)?;
        Ok(sig)
    }
}