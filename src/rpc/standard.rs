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
    // Extract transaction data from params
    let tx_data = params.get(0)
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing transaction data parameter".to_string())?;
    
    // Send transaction
    let signature = manager.send_transaction(fork_id, tx_data).await?;
    
    // Return signature
    Ok(json!(signature))
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

pub async fn handle_get_latest_blockhash(
    manager: &ForkManager,
    fork_id: &str,
    _params: &Value,
) -> Result<Value, String> {
    let blockhash = manager.get_latest_blockhash(fork_id).await?;
    
    Ok(json!({
        "blockhash": blockhash.to_string(),
        "lastValidBlockHeight": 999999999  // Dummy value for testing
    }))
}