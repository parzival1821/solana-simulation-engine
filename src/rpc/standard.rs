use crate::fork_manager::ForkManager;
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Handle getBalance RPC method
pub async fn handle_get_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // Extract address from params array
    let address = params.get(0)
        .and_then(|a| a.as_str())
        .ok_or_else(|| "Missing address parameter".to_string())?;
    
    // Get balance from fork manager
    let balance = manager.get_balance(fork_id, address).await?;
    
    // Return result
    Ok(json!(balance))
}

/// Handle sendTransaction RPC method (TODO for later)
pub async fn handle_send_transaction(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // TODO: Implement later
    Err("sendTransaction not yet implemented".to_string())
}

/// Handle getAccountInfo RPC method (TODO for later)
pub async fn handle_get_account_info(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // TODO: Implement later
    Err("getAccountInfo not yet implemented".to_string())
}