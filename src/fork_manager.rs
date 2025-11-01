use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use litesvm :: LiteSVM;
use std::time::{Instant,Duration};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;
use std::str::FromStr;
use solana_system_interface::program as system_program;

// TODO: What should we store for each fork?
// For now, just store a String (placeholder)
// Later we'll replace this with LiteSVM

struct Fork {
    svm : Arc<RwLock<LiteSVM>>,
    timestamp : Instant
}

// Define the storage type
type ForkStorage = Arc<RwLock<HashMap<String, Fork>>>;

// The manager struct
#[derive(Clone)]
pub struct ForkManager {
    forks: ForkStorage,
}

impl ForkManager {
    // Constructor
    pub fn new() -> Self {
         let manager = Self {
            forks: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start cleanup task
        manager.start_cleanup_task();
        
        manager
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
                
                println!("🧹 Cleanup: {} forks remaining", forks_map.len());
            }
        });
    }
    
    // Create a new fork
    pub async fn create_fork(&self) -> String {
        // TODO: 
        // 1. Generate a UUID
        let uid = Uuid::new_v4().to_string();
        // 2. Lock the HashMap for writing
        let mut forks = self.forks.write().await;
        // 3. Insert a new entry
        let mut fork = Fork{
            svm : Arc::new(RwLock::new(LiteSVM::new())),
            timestamp : Instant::now()
        };
        forks.insert(uid.clone(), fork);
        // 4. Return the fork ID
        uid
    }

    pub async fn get_balance(&self, fork_id: &str, address: &str) -> Result<u64, String> {
        let forks = self.forks.read().await;
        
        let fork = forks.get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        
        let pubkey = Pubkey::from_str(address)
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let svm = fork.svm.read().await;
        let balance = svm.get_balance(&pubkey)
            .ok_or_else(|| format!("Failed to get balance for {:?}", pubkey))?;
        
        Ok(balance)
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
        
        // Set the balance (not add!)
        account.lamports = lamports;
        
        // Write back
        svm.set_account(pubkey, account);
        
        Ok(())
    }

    pub async fn send_transaction(&self, fork_id: &str, tx_data: &str) -> Result<(), String> {
        let forks = self.forks.read().await;

        let fork = forks.get(fork_id)
                    .ok_or_else(|| format!("Fork not found : {}", fork_id))?;

        let mut svm = fork.svm.write().await;

        // Decode transaction from base64/base58
        // Send it to the SVM
        // Return signature
    }
}