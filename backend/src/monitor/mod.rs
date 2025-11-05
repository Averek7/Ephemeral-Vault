use solana_client::rpc_client::RpcClient;
use anyhow::Result;
use tokio::time::{sleep, Duration};
use crate::db;
use crate::auto_deposit::calculator::AutoDepositCalculator;

pub struct VaultMonitor {
    rpc: RpcClient,
    poll_interval: Duration,
    auto_calc: AutoDepositCalculator,
    // notification sender (webhook, slack, etc.)
}

impl VaultMonitor {
    pub fn new(rpc_url: &str, poll_interval_secs: u64) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url.to_string()),
            poll_interval: Duration::from_secs(poll_interval_secs),
            auto_calc: AutoDepositCalculator::new(rpc_url.to_string()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let sessions = db::fetch_active_sessions()?; // your DB of sessions
            for s in sessions {
                let balance = self.rpc.get_balance(&s.vault)?;
                // low balance action
                if self.auto_calc.needs_top_up(&s.vault, /* threshold */ 10000)? {
                    // enqueue topup job or call auto-deposit flow
                    println!("Vault {} low balance: {}", s.vault, balance);
                    db::enqueue_topup_job(&s)?;
                }
                // expiry detection
                if s.expires_at <= chrono::Utc::now().timestamp() {
                    db::enqueue_cleanup_job(&s)?;
                }
                // anomaly detection — add rules for > X lamports change
            }
            sleep(self.poll_interval).await;
        }
    }
}
