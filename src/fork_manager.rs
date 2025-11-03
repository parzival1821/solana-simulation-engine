use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use litesvm::LiteSVM;
use std::time::{Instant,Duration};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::transaction::Transaction;
use solana_sdk::hash::Hash;
use std::str::FromStr;
use solana_system_interface::program as system_program;
use bincode;
use solana_client::rpc_client::RpcClient; 
use chrono;
use serde::Serialize;

use dotenv::dotenv;
use std::env;

struct Fork {
    svm : Arc<RwLock<LiteSVM>>,
    timestamp : Instant,
    pub transaction_history: Arc<RwLock<Vec<TransactionRecord>>>,
}

#[derive(Clone, Serialize)]
pub struct TransactionRecord {
    pub signature: String,
    pub timestamp: String,
    pub success: bool,
}

// Define the storage type
type ForkStorage = Arc<RwLock<HashMap<String, Fork>>>;

#[derive(Clone)]
pub struct ForkManager {
    forks: ForkStorage,
    rpc_client : Arc<RpcClient>
}

impl ForkManager {
    pub fn new() -> Self {

        dotenv().ok();
        let mainnet_api = env::var("MAINNET_HELIUS")
                            .expect("Devnet api key must be set in .env");

        let mainnet_default="https://api.mainnet-beta.solana.com";
        
         let manager = Self {
            forks: Arc::new(RwLock::new(HashMap::new())),
            rpc_client : Arc::new(RpcClient::new(mainnet_default.to_string())),
        };
        
        // Start cleanup task
        manager.start_cleanup_task();
        
        manager
    }

    pub async fn ensure_account_exists(&self, fork_id: &str, pubkey: &Pubkey) -> Result<(), String> {
        let forks = self.forks.read().await;
        let fork = forks.get(fork_id).ok_or("Fork not found")?;
        
        let mut svm = fork.svm.write().await;
        
        // Check if account exists
        if svm.get_account(pubkey).is_none() {
            // Fetch from mainnet
            println!("🔍 Fetching account {} from mainnet...", pubkey);
            
            let account = self.rpc_client
                .get_account(pubkey)
                .map_err(|e| format!("Failed to fetch account: {}", e))?;
            
            // Load into fork
            svm.set_account(*pubkey, account);
        }
        
        Ok(())
    }

    fn start_cleanup_task(&self) {
        let forks = Arc::clone(&self.forks);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await; // wait for 60 secs on this line 
                
                // Clean up expired forks
                let mut forks_map = forks.write().await;
                let now = Instant::now();
                
                forks_map.retain(|_id, fork| {
                    let age = now.duration_since(fork.timestamp);
                    age < Duration::from_secs(900) // 15 mins
                });
                
                println!("Cleanup: {} forks remaining", forks_map.len());
            }
        });
    }
    
    // Create a new fork
    pub async fn create_fork(&self) -> String {
        let uid = Uuid::new_v4().to_string();
        let mut forks = self.forks.write().await;
        let mut fork = Fork{
            svm : Arc::new(RwLock::new(LiteSVM::new())),
            timestamp : Instant::now(),
            transaction_history : Arc::new(RwLock::new(Vec::new())),
        };
        forks.insert(uid.clone(), fork);
        uid
    }

    pub async fn get_balance(&self, fork_id: &str, address: &str) -> Result<u64, String> {
        let forks = self.forks.read().await;
        
        let fork = forks.get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let mut svm = fork.svm.write().await;
        
        // Check if account exists locally first
        if let Some(account) = svm.get_account(&pubkey) {
            return Ok(account.lamports);
        }
        
        // Not found locally - try fetching from mainnet
        drop(svm);  // Drop write lock before calling another method
        drop(forks); // Drop read lock too
        
        println!("🔍 Account not in fork, fetching from mainnet...");
        
        // Fetch from mainnet (this will cache it)
        if let Some(account) = self.get_account_info(fork_id, address).await? {
            Ok(account.lamports)
        } else {
            // Account doesn't exist on mainnet either - return 0
            Ok(0)
        }
    }

    pub async fn set_balance(&self, fork_id: &str, address: &str, lamports: u64) -> Result<(), String> {
        let forks = self.forks.read().await;
        
        let fork = forks.get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let mut svm = fork.svm.write().await;
        
        // Get existing account or create new one
        let mut account = svm.get_account(&pubkey)
            .unwrap_or_else(|| Account {
                lamports: 0,
                data: vec![],
                owner: system_program::id(),
                executable: false,
                rent_epoch: 0,
            });
        
        account.lamports = lamports;
        
        // Write back
        svm.set_account(pubkey, account);
        
        Ok(())
    }

    pub async fn send_transaction(&self, fork_id: &str, tx_data: &str) -> Result<String, String> {
        let forks = self.forks.read().await;

        let fork = forks.get(fork_id)
                    .ok_or_else(|| format!("Fork not found : {}", fork_id))?;

        let mut svm = fork.svm.write().await;

        // Decode transaction from base64/base58
        let decoded = bs58::decode(tx_data)
                        .into_vec()
                        .map_err(|e| format!("Error in decoding tx to base 58 : {}", e))?;
        
        let tx : Transaction = bincode::deserialize(&decoded)
                        .map_err(|e| format!("Error in deserializing the tx : {}", e))?;

        // Send it to the SVM
        let result = svm.send_transaction(tx);
        
        // Return signature
        // Ok(result.signature.to_string())

        match result {
            Ok(metadata) => {
                let sig = metadata.signature.to_string();
                let mut history = fork.transaction_history.write().await;
                history.push(TransactionRecord{
                    signature : sig.clone(),
                    timestamp : chrono::Local::now().to_rfc3339(),
                    success : true,
                });

                Ok(sig)
            }
            Err(e) => {
                let mut history = fork.transaction_history.write().await;
                history.push(TransactionRecord{
                    signature : "failed".to_string(),
                    timestamp : chrono::Local::now().to_rfc3339(),
                    success : false,
                });

                Err(format!("Transaction failed : {:?}",e))
            }
        }

        // match result {
        //     Ok(metadata) => {
        //         // Extract signature from transaction metadata
        //         let signature = metadata.signature.to_string();
                
        //         // Record transaction
        //         if let Some(history) = fork.transaction_history.write().await {
        //             let mut hist = history.write().await;
        //             hist.push(TransactionRecord {
        //                 signature: signature.clone(),
        //                 timestamp: chrono::Local::now().to_rfc3339(),
        //                 success: true,
        //             });
        //         }
                
        //         println!("✅ Transaction executed: {}", signature);
        //         Ok(signature)
        //     }
        //     Err(e) => {
        //         // Record failure
        //         if let Some(history) = fork.transaction_history.as_ref() {
        //             let mut hist = history.write().await;
        //             hist.push(TransactionRecord {
        //                 signature: "failed".to_string(),
        //                 timestamp: chrono::Local::now().to_rfc3339(),
        //                 success: false,
        //             });
        //         }
                
        //         // Format error properly
        //         Err(format!("Transaction failed: {:?}", e))
        //     }
        // }
    }

    pub async fn get_transaction_history(&self, fork_id: &str) -> Result<Vec<TransactionRecord>, String> {
        let forks = self.forks.read().await;
        let fork = forks.get(fork_id)
            .ok_or_else(|| "Fork not found".to_string())?;
        
        let history = fork.transaction_history.read().await;
        Ok(history.clone())
    }

    pub async fn get_latest_blockhash(&self, fork_id: &str) -> Result<Hash, String> {
        let forks = self.forks.read().await;
        
        let fork = forks.get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        
        let svm = fork.svm.read().await;
        Ok(svm.latest_blockhash())
    }

    pub async fn get_account_info(&self, fork_id: &str, address: &str) -> Result<Option<Account>, String> {
        let forks = self.forks.read().await;
        
        let fork = forks.get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let mut svm = fork.svm.write().await;
        
        // First check if account exists locally
        if let Some(account) = svm.get_account(&pubkey) {
            return Ok(Some(account));
        }
        
        // If not found locally, fetch from mainnet
        println!("🔍 Fetching account {} from mainnet...", pubkey);
        
        match self.rpc_client.get_account(&pubkey) {
            Ok(account) => {
                // Cache it in the fork
                svm.set_account(pubkey, account.clone());
                println!("✓ Account loaded from mainnet and cached");
                Ok(Some(account))
            }
            Err(e) => {
                println!("⚠️  Account not found on mainnet: {:#?}", e);
                Ok(None)
            }
        }
    }
}