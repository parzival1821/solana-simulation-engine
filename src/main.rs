mod fork_manager;
mod rpc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use fork_manager::ForkManager;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let manager = ForkManager::new();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/fork/create", post(create_fork))
        .route("/fork/{fork_id}/rpc", post(handle_rpc))
        .route("/fork/{fork_id}/transactions", get(get_transaction_history))
        .with_state(manager);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

// State(manager) extracts the shared ForkManager from the router
async fn create_fork(State(manager): State<ForkManager>) -> Json<Value> {
    // Call the manager to create a new fork
    let fork_id = manager.create_fork().await;

    // Return the fork ID as JSON
    Json(json!({
        "fork_id": fork_id
    }))
}

async fn get_transaction_history(
    Path(fork_id): Path<String>,
    State(manager): State<ForkManager>,
) -> Json<Value> {
    match manager.get_transaction_history(&fork_id).await {
        Ok(history) => Json(json!({
            "transactions" : history
        })),
        Err(e) => Json(json!({
            "error" : e
        })),
    }
}

async fn handle_rpc(
    Path(fork_id): Path<String>,
    State(manager): State<ForkManager>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let method = payload
        .get("method") // extract "method" field from JSON
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let id = payload.get("id").cloned().unwrap_or(json!(1));
    let params = payload.get("params").cloned().unwrap_or(json!([]));

    // Route to appropriate handler based on method
    let result = match method {
        // Standard RPC methods
        "getBalance" => rpc::standard::handle_get_balance(&manager, &fork_id, &params).await,
        "sendTransaction" => {
            rpc::standard::handle_send_transaction(&manager, &fork_id, &params).await
        }
        "getLatestBlockhash" => {
            rpc::standard::handle_get_latest_blockhash(&manager, &fork_id, &params).await
        }
        "getAccountInfo" => {
            rpc::standard::handle_get_account_info(&manager, &fork_id, &params).await
        }
        "get_token_balance" => {
            rpc::standard::handle_get_token_balance(&manager, &fork_id, &params).await
        }

        // Cheatcode methods
        "set_balance" => rpc::cheatcodes::handle_set_balance(&manager, &fork_id, &params).await,
        "set_token_balance" => {
            rpc::cheatcodes::handle_set_token_balance(&manager, &fork_id, &params).await
        }

        _ => Err(format!("Method not found: {}", method)),
    };

    // Format response
    match result {
        Ok(value) => Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value
        })),
        Err(err) => Json(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32000,
                "message": err
            }
        })),
    }
}
