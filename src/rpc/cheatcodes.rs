use crate::fork_manager::ForkManager;
use serde_json::{json, Value};

/// Handle set_balance cheatcode
pub async fn handle_set_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // Extract address and lamports from params object
    let address = params
        .get("address")
        .and_then(|a| a.as_str())
        .ok_or_else(|| "Missing address parameter".to_string())?;

    let lamports = params
        .get("lamports")
        .and_then(|l| l.as_u64())
        .ok_or_else(|| "Missing or invalid lamports parameter".to_string())?;

    // Call fork manager
    manager.set_balance(fork_id, address, lamports).await?;

    // Return success
    Ok(json!("Success"))
}

pub async fn handle_set_token_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    let owner = params
        .get("owner")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing owner parameter".to_string())?;

    let mint = params
        .get("mint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing mint parameter".to_string())?;

    let amount = params
        .get("amount")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "Missing or invalid amount parameter".to_string())?;

    manager
        .set_token_balance(fork_id, owner, mint, amount)
        .await?;

    Ok(json!("Success"))
}
