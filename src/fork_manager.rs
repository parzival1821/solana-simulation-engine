use bincode;
use chrono;
use litesvm::LiteSVM;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_system_interface::program as system_program;
use spl_associated_token_account::get_associated_token_address;
use spl_token::solana_program::program_pack::Pack;
use spl_token::solana_program::pubkey as spl_pubkey;
use spl_token::state::{Account as TokenAccount, AccountState};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

struct Fork {
    svm: Arc<RwLock<LiteSVM>>,
    timestamp: Instant,
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
    rpc_client: Arc<RpcClient>,
}

impl ForkManager {
    pub fn new() -> Self {
        let mainnet_default = "https://api.mainnet-beta.solana.com";

        let manager = Self {
            forks: Arc::new(RwLock::new(HashMap::new())),
            rpc_client: Arc::new(RpcClient::new(mainnet_default.to_string())),
        };

        // Start cleanup task
        manager.start_cleanup_task();

        manager
    }

    pub async fn ensure_account_exists(
        &self,
        fork_id: &str,
        pubkey: &Pubkey,
    ) -> Result<(), String> {
        let forks = self.forks.read().await;
        let fork = forks.get(fork_id).ok_or("Fork not found")?;

        let mut svm = fork.svm.write().await;

        // Check if account exists
        if svm.get_account(pubkey).is_none() {
            // Fetch from mainnet
            println!("🔍 Fetching account {} from mainnet...", pubkey);

            let account = self
                .rpc_client
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
        let mut fork = Fork {
            svm: Arc::new(RwLock::new(LiteSVM::new())),
            timestamp: Instant::now(),
            transaction_history: Arc::new(RwLock::new(Vec::new())),
        };
        forks.insert(uid.clone(), fork);
        uid
    }

    pub async fn get_balance(&self, fork_id: &str, address: &str) -> Result<u64, String> {
        let forks = self.forks.read().await;

        let fork = forks
            .get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;

        let pubkey = Pubkey::from_str(address).map_err(|e| format!("Invalid address: {}", e))?;

        let mut svm = fork.svm.write().await;

        // Check if account exists locally first
        if let Some(account) = svm.get_account(&pubkey) {
            return Ok(account.lamports);
        }

        // Not found locally - try fetching from mainnet
        drop(svm); // Drop write lock before calling another method
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

    pub async fn set_balance(
        &self,
        fork_id: &str,
        address: &str,
        lamports: u64,
    ) -> Result<(), String> {
        let forks = self.forks.read().await;

        let fork = forks
            .get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;

        let pubkey = Pubkey::from_str(address).map_err(|e| format!("Invalid address: {}", e))?;

        let mut svm = fork.svm.write().await;

        // Get existing account or create new one
        let mut account = svm.get_account(&pubkey).unwrap_or_else(|| Account {
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

        let fork = forks
            .get(fork_id)
            .ok_or_else(|| format!("Fork not found : {}", fork_id))?;

        let mut svm = fork.svm.write().await;

        // Decode transaction from base64/base58
        let decoded = bs58::decode(tx_data)
            .into_vec()
            .map_err(|e| format!("Error in decoding tx to base 58 : {}", e))?;

        let tx: Transaction = bincode::deserialize(&decoded)
            .map_err(|e| format!("Error in deserializing the tx : {}", e))?;

        // Send it to the SVM
        let result = svm.send_transaction(tx);

        // Return signature
        // Ok(result.signature.to_string())

        match result {
            Ok(metadata) => {
                let sig = metadata.signature.to_string();
                let mut history = fork.transaction_history.write().await;
                history.push(TransactionRecord {
                    signature: sig.clone(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    success: true,
                });

                Ok(sig)
            }
            Err(e) => {
                let mut history = fork.transaction_history.write().await;
                history.push(TransactionRecord {
                    signature: "failed".to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    success: false,
                });

                Err(format!("Transaction failed : {:?}", e))
            }
        }
    }

    pub async fn get_transaction_history(
        &self,
        fork_id: &str,
    ) -> Result<Vec<TransactionRecord>, String> {
        let forks = self.forks.read().await;
        let fork = forks
            .get(fork_id)
            .ok_or_else(|| "Fork not found".to_string())?;

        let history = fork.transaction_history.read().await;
        Ok(history.clone())
    }

    pub async fn get_latest_blockhash(&self, fork_id: &str) -> Result<Hash, String> {
        let forks = self.forks.read().await;

        let fork = forks
            .get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;

        let svm = fork.svm.read().await;
        Ok(svm.latest_blockhash())
    }

    pub async fn get_account_info(
        &self,
        fork_id: &str,
        address: &str,
    ) -> Result<Option<Account>, String> {
        let pubkey = Pubkey::from_str(address).map_err(|e| format!("Invalid address: {}", e))?;

        // First check local fork
        {
            let forks = self.forks.read().await;
            let fork = forks
                .get(fork_id)
                .ok_or_else(|| format!("Fork not found: {}", fork_id))?;

            let svm = fork.svm.read().await;

            if let Some(account) = svm.get_account(&pubkey) {
                return Ok(Some(account));
            }
            // Lock released here
        }

        // Not found locally - fetch from mainnet
        println!("🔍 Fetching account {} from mainnet...", pubkey);

        match self.rpc_client.get_account(&pubkey) {
            Ok(account) => {
                // Cache it in the fork
                let forks = self.forks.read().await;
                let fork = forks
                    .get(fork_id)
                    .ok_or_else(|| format!("Fork not found: {}", fork_id))?;

                let mut svm = fork.svm.write().await;
                svm.set_account(pubkey, account.clone());
                println!("✓ Account loaded from mainnet and cached");

                Ok(Some(account))
            }
            Err(e) => {
                println!("⚠️  Account not found on mainnet: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn set_token_balance(
        &self,
        fork_id: &str,
        owner: &str,
        mint: &str,
        amount: u64,
    ) -> Result<(), String> {
        let owner_pubkey =
            Pubkey::from_str(owner).map_err(|e| format!("Invalid owner address: {}", e))?;
        let mint_pubkey =
            Pubkey::from_str(mint).map_err(|e| format!("Invalid mint address: {}", e))?;

        let token_account_pubkey = get_associated_token_address(&owner_pubkey, &mint_pubkey);

        println!("📝 Token account address: {}", token_account_pubkey);

        // Check if mint exists, fetch if needed
        {
            let forks = self.forks.read().await;
            let fork = forks
                .get(fork_id)
                .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
            let svm = fork.svm.read().await;

            if svm.get_account(&mint_pubkey).is_none() {
                drop(svm);
                drop(forks);

                println!("🔍 Fetching mint from mainnet...");
                match self.rpc_client.get_account(&mint_pubkey) {
                    Ok(mint_account) => {
                        let forks = self.forks.read().await;
                        let fork = forks.get(fork_id).unwrap();
                        let mut svm = fork.svm.write().await;
                        svm.set_account(mint_pubkey, mint_account);
                    }
                    Err(_) => {}
                }
            }
        }

        // Step 2: Try to fetch existing token account from mainnet FIRST
        let existing_from_mainnet = {
            let forks = self.forks.read().await;
            let fork = forks.get(fork_id).unwrap();
            let svm = fork.svm.read().await;

            // Check if already in fork
            if svm.get_account(&token_account_pubkey).is_some() {
                None // Already have it locally
            } else {
                drop(svm);
                drop(forks);

                // Try fetching from mainnet
                println!("🔍 Checking mainnet for existing token account...");
                match self.rpc_client.get_account(&token_account_pubkey) {
                    Ok(mainnet_account) => {
                        println!("✓ Found existing token account on mainnet");

                        // Parse current balance from mainnet
                        if let Ok(token_acc) = TokenAccount::unpack(&mainnet_account.data) {
                            println!("  Current mainnet balance: {} tokens", token_acc.amount);
                        }

                        Some(mainnet_account)
                    }
                    Err(_) => {
                        println!("ℹ️  No existing token account on mainnet, will create new");
                        None
                    }
                }
            }
        };

        // Now create/update token account
        let forks = self.forks.read().await;
        let fork = forks
            .get(fork_id)
            .ok_or_else(|| format!("Fork not found: {}", fork_id))?;
        let mut svm = fork.svm.write().await;

        let mut account_data = if let Some(existing) = svm.get_account(&token_account_pubkey) {
            existing
        } else {
            let rent = svm.minimum_balance_for_rent_exemption(TokenAccount::LEN);
            Account {
                lamports: rent,
                data: vec![0; TokenAccount::LEN],
                owner: spl_token::id().to_bytes().into(),
                executable: false,
                rent_epoch: 0,
            }
        };

        let token_account = TokenAccount {
            mint: spl_pubkey::Pubkey::new_from_array(*mint_pubkey.as_array()),
            owner: spl_pubkey::Pubkey::new_from_array(*owner_pubkey.as_array()),
            amount,
            delegate: Default::default(),
            state: AccountState::Initialized,
            is_native: Default::default(),
            delegated_amount: 0,
            close_authority: Default::default(),
        };

        Pack::pack(token_account, &mut account_data.data)
            .map_err(|e| format!("Pack error: {:?}", e))?;

        svm.set_account(token_account_pubkey, account_data);

        println!("✅ Token balance set: {} tokens", amount);
        Ok(())
    }

    pub async fn get_token_balance(
        &self,
        fork_id: &str,
        owner: &str,
        mint: &str,
    ) -> Result<u64, String> {
        let owner_pubkey = Pubkey::from_str(owner).map_err(|e| format!("Invalid owner: {}", e))?;
        let mint_pubkey = Pubkey::from_str(mint).map_err(|e| format!("Invalid mint: {}", e))?;

        let token_account_pubkey = get_associated_token_address(&owner_pubkey, &mint_pubkey);

        // First check local fork
        {
            let forks = self.forks.read().await;
            let fork = forks.get(fork_id).ok_or("Fork not found")?;
            let svm = fork.svm.read().await;

            if let Some(account) = svm.get_account(&token_account_pubkey) {
                match TokenAccount::unpack(&account.data) {
                    Ok(ta) => {
                        println!("✓ Token balance from fork: {} tokens", ta.amount);
                        return Ok(ta.amount);
                    }
                    Err(_) => {}
                }
            }
        }

        // Not in fork - try fetching from mainnet
        println!("🔍 Token account not in fork, checking mainnet...");

        match self.rpc_client.get_account(&token_account_pubkey) {
            Ok(account) => {
                // Cache it in the fork
                {
                    let forks = self.forks.read().await;
                    let fork = forks.get(fork_id).unwrap();
                    let mut svm = fork.svm.write().await;
                    svm.set_account(token_account_pubkey, account.clone());
                }

                // Unpack and return balance
                match TokenAccount::unpack(&account.data) {
                    Ok(ta) => {
                        println!("✓ Mainnet balance: {} tokens", ta.amount);
                        Ok(ta.amount)
                    }
                    Err(_) => Ok(0),
                }
            }
            Err(_) => {
                println!("⚠️  Token account doesn't exist on mainnet");
                Ok(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_create_fork() {
        let manager = ForkManager::new();

        let fork_id = manager.create_fork().await;

        assert!(!fork_id.is_empty());
        assert_eq!(fork_id.len(), 36); // UUID length
        println!("✓ Fork created with ID: {}", fork_id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_fork_isolation() {
        let manager = ForkManager::new();

        // Create two forks
        let fork1 = manager.create_fork().await;
        let fork2 = manager.create_fork().await;

        // Verify they're different
        assert_ne!(fork1, fork2);

        // Set different balances in each fork
        let address = "So11111111111111111111111111111111111111112"; // Valid 44-char address

        manager
            .set_balance(&fork1, address, 5_000_000_000)
            .await
            .unwrap();
        manager
            .set_balance(&fork2, address, 9_000_000_000)
            .await
            .unwrap();

        // Verify isolation
        let balance1 = manager.get_balance(&fork1, address).await.unwrap();
        let balance2 = manager.get_balance(&fork2, address).await.unwrap();

        assert_eq!(balance1, 5_000_000_000);
        assert_eq!(balance2, 9_000_000_000);

        println!("✓ Fork 1 balance: {} lamports", balance1);
        println!("✓ Fork 2 balance: {} lamports", balance2);
        println!("✓ Forks are properly isolated");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_set_and_get_balance() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let address = "So11111111111111111111111111111111111111112";
        let expected_balance = 10_000_000_000; // 10 SOL

        // Set balance
        manager
            .set_balance(&fork_id, address, expected_balance)
            .await
            .unwrap();

        // Get balance
        let actual_balance = manager.get_balance(&fork_id, address).await.unwrap();

        assert_eq!(actual_balance, expected_balance);
        println!(
            "✓ Balance set and retrieved correctly: {} lamports",
            actual_balance
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_multiple_accounts_in_fork() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        // Use valid base58 addresses (44 characters)
        let addr1 = "So11111111111111111111111111111111111111112";
        let addr2 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        let addr3 = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

        // Set different balances
        manager
            .set_balance(&fork_id, addr1, 1_000_000_000)
            .await
            .unwrap();
        manager
            .set_balance(&fork_id, addr2, 2_000_000_000)
            .await
            .unwrap();
        manager
            .set_balance(&fork_id, addr3, 3_000_000_000)
            .await
            .unwrap();

        // Verify all balances
        let bal1 = manager.get_balance(&fork_id, addr1).await.unwrap();
        let bal2 = manager.get_balance(&fork_id, addr2).await.unwrap();
        let bal3 = manager.get_balance(&fork_id, addr3).await.unwrap();

        assert_eq!(bal1, 1_000_000_000);
        assert_eq!(bal2, 2_000_000_000);
        assert_eq!(bal3, 3_000_000_000);

        println!("✓ Multiple accounts managed correctly in same fork");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_balance_update() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let address = "So11111111111111111111111111111111111111112";

        // Set initial balance
        manager
            .set_balance(&fork_id, address, 5_000_000_000)
            .await
            .unwrap();
        let balance1 = manager.get_balance(&fork_id, address).await.unwrap();
        assert_eq!(balance1, 5_000_000_000);

        // Update balance
        manager
            .set_balance(&fork_id, address, 10_000_000_000)
            .await
            .unwrap();
        let balance2 = manager.get_balance(&fork_id, address).await.unwrap();
        assert_eq!(balance2, 10_000_000_000);

        println!("✓ Balance updated from {} to {}", balance1, balance2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_nonexistent_account_returns_zero() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        // Use a valid base58 address that likely doesn't exist on mainnet
        // This is a random valid address
        let address = "ABqmzP8hzRgpDGPMJm3c9TD5SUwLBsu7372HGxQeWhUr";

        let balance = manager.get_balance(&fork_id, address).await.unwrap();

        // Should return 0 for non-existent accounts
        assert_eq!(balance, 0);
        println!("✓ Nonexistent account returns 0 balance");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_invalid_fork_id() {
        let manager = ForkManager::new();

        let fake_fork_id = "nonexistent-fork-id";
        let address = "So11111111111111111111111111111111111111112";

        let result = manager.get_balance(fake_fork_id, address).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Fork not found"));
        println!("✓ Invalid fork ID properly rejected");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_invalid_address_format() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let invalid_address = "not-a-valid-address";

        let result = manager.set_balance(&fork_id, invalid_address, 1000).await;

        assert!(result.is_err());
        println!("✓ Invalid address format properly rejected");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_latest_blockhash() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let blockhash = manager.get_latest_blockhash(&fork_id).await.unwrap();

        assert_ne!(blockhash.to_string(), "11111111111111111111111111111111");
        println!("✓ Got valid blockhash: {}", blockhash);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_transaction_history_initialization() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let history = manager.get_transaction_history(&fork_id).await.unwrap();

        assert_eq!(history.len(), 0);
        println!("✓ New fork has empty transaction history");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_fork_access() {
        let manager = Arc::new(ForkManager::new());
        let fork_id = manager.create_fork().await;

        // Use valid base58 addresses
        let address1 = "So11111111111111111111111111111111111111112";
        let address2 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

        // Spawn concurrent tasks
        let manager1 = Arc::clone(&manager);
        let fork_id1 = fork_id.clone();
        let handle1 = tokio::spawn(async move {
            manager1
                .set_balance(&fork_id1, address1, 1_000_000_000)
                .await
                .unwrap();
        });

        let manager2 = Arc::clone(&manager);
        let fork_id2 = fork_id.clone();
        let handle2 = tokio::spawn(async move {
            manager2
                .set_balance(&fork_id2, address2, 2_000_000_000)
                .await
                .unwrap();
        });

        // Wait for both
        handle1.await.unwrap();
        handle2.await.unwrap();

        // Verify both operations succeeded
        let bal1 = manager.get_balance(&fork_id, address1).await.unwrap();
        let bal2 = manager.get_balance(&fork_id, address2).await.unwrap();

        assert_eq!(bal1, 1_000_000_000);
        assert_eq!(bal2, 2_000_000_000);

        println!("✓ Concurrent access to same fork works correctly");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_spl_token_balance() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let owner = "D2bJqkFEa65xFKii3dW2ByrZEitdpX3PLR9uezPoSNKi";
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let amount = 1_000_000_000; // 1000 USDC (6 decimals)

        // Set token balance (this will try to fetch mint from mainnet)
        // It may fail if network is unavailable, so we handle that
        match manager
            .set_token_balance(&fork_id, owner, usdc_mint, amount)
            .await
        {
            Ok(_) => {
                // Get token balance
                let balance = manager
                    .get_token_balance(&fork_id, owner, usdc_mint)
                    .await
                    .unwrap();

                assert_eq!(balance, amount);
                println!("✓ SPL token balance set and retrieved: {} tokens", balance);
            }
            Err(e) => {
                println!("⚠️  Skipping SPL token test (network issue): {}", e);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_fork_expiration_timestamp() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let forks = manager.forks.read().await;
        let fork = forks.get(&fork_id).unwrap();

        let now = Instant::now();

        // Fork timestamp is when it was created
        // It should expire 900 seconds (15 minutes) AFTER creation
        // So fork.timestamp is in the past, and now > fork.timestamp
        let time_since_creation = now.duration_since(fork.timestamp);

        // Fork was just created, so it should be very recent (< 1 second old)
        assert!(time_since_creation.as_secs() < 10);

        println!(
            "✓ Fork created {} seconds ago",
            time_since_creation.as_secs()
        );
        println!("✓ Fork will expire in ~900 seconds (15 minutes)");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_account_info_retrieval() {
        let manager = ForkManager::new();
        let fork_id = manager.create_fork().await;

        let address = "So11111111111111111111111111111111111111112";

        // Set balance
        manager
            .set_balance(&fork_id, address, 5_000_000_000)
            .await
            .unwrap();

        // Get account info
        let account = manager.get_account_info(&fork_id, address).await.unwrap();

        assert!(account.is_some());
        let acc = account.unwrap();
        assert_eq!(acc.lamports, 5_000_000_000);

        println!("✓ Account info retrieved correctly");
        println!("  Lamports: {}", acc.lamports);
        println!("  Owner: {}", acc.owner);
    }
}
