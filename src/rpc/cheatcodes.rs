use crate::fork_manager::ForkManager;
use serde_json::{json, Value};

/// Handle set_balance cheatcode
pub async fn handle_set_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // Extract address and lamports from params object
    let address = params.get("address")
        .and_then(|a| a.as_str())
        .ok_or_else(|| "Missing address parameter".to_string())?;
    
    let lamports = params.get("lamports")
        .and_then(|l| l.as_u64())
        .ok_or_else(|| "Missing or invalid lamports parameter".to_string())?;
    
    // Call fork manager
    manager.set_balance(fork_id, address, lamports).await?;
    
    // Return success
    Ok(json!("Success"))
}

/// Handle set_token_balance cheatcode (TODO for later)
pub async fn handle_set_token_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // TODO: Implement later
    Err("set_token_balance not yet implemented".to_string())
}