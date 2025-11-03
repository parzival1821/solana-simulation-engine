use crate::fork_manager::ForkManager;
use serde_json::{json, Value};
/// Handle getBalance RPC method
pub async fn handle_get_balance(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    // Extract address from params array
    let address = params
        .get(0)
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
    let tx_data = params
        .get(0)
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing transaction data parameter".to_string())?;

    // Send transaction
    let signature = manager.send_transaction(fork_id, tx_data).await?;

    // Return signature
    Ok(json!(signature))
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

pub async fn handle_get_account_info(
    manager: &ForkManager,
    fork_id: &str,
    params: &Value,
) -> Result<Value, String> {
    let address = params
        .get(0)
        .and_then(|a| a.as_str())
        .ok_or_else(|| "Missing address parameter".to_string())?;

    let account = manager.get_account_info(fork_id, address).await?;

    match account {
        Some(acc) => {
            // Encode data as base58
            let data_base58 = bs58::encode(&acc.data).into_string();

            Ok(json!({
                "value": {
                    "lamports": acc.lamports,
                    "owner": acc.owner.to_string(),
                    "data": [data_base58, "base58"],
                    "executable": acc.executable,
                    "rentEpoch": acc.rent_epoch
                }
            }))
        }
        None => Ok(json!({
            "value": null
        })),
    }
}

pub async fn handle_get_token_balance(
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

    let balance = manager.get_token_balance(fork_id, owner, mint).await?;

    Ok(json!(balance))
}
