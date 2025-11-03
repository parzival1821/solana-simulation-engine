# Solana Fork Engine

A lightweight Solana simulation engine that creates isolated network forks for testing dApp interactions without touching mainnet. Built with [liteSVM](https://github.com/LiteSVM/litesvm) for fast, in-memory Solana blockchain simulation.

## Overview

This engine allows developers to:
- Create isolated forks of Solana mainnet state
- Test protocol interactions safely without spending real funds
- Manipulate account balances (SOL and SPL tokens) instantly
- Execute and simulate transactions in a controlled environment
- Automatically fetch missing accounts from mainnet on-demand

Similar to how [Tenderly](https://tenderly.co) provides Ethereum mainnet forks, this engine brings the same capability to Solana.

---

## Features

### Core Functionality
- ✅ **Mainnet Fork Creation** - Fork from latest Solana state using liteSVM
- ✅ **Fork Isolation** - Each user gets independent fork environment
- ✅ **Auto-Expiration** - Forks automatically cleanup after 15 minutes
- ✅ **SOL Balance Manipulation** - Instantly set account balances
- ✅ **SPL Token Support** - Set and query token balances
- ✅ **Transaction Execution** - Execute real Solana transactions
- ✅ **Mainnet Account Fetching** - Auto-fetch missing accounts from mainnet
- ✅ **Transaction History** - Track all executed transactions per fork
- ✅ **JSON-RPC API** - Standard Solana RPC + custom cheatcodes

### Advanced Features
- **On-Demand State Loading** - Accounts fetched from mainnet only when needed
- **Concurrent Fork Support** - Multiple isolated forks running simultaneously
- **Thread-Safe Operations** - Safe concurrent access to fork state
- **Standard RPC Compliance** - Compatible with existing Solana tooling

---

## Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- `jq` (for running test scripts)

### Installation
```bash
git clone https://github.com/parzival1821/solana-simulation-engine
cd solana-simulation-engine
cargo build --release
```

### Run Server
```bash
cargo run
# Server starts on http://localhost:3000
```

### Run Tests

**Unit Tests:**
```bash
cargo test -- --nocapture
```

**Integration Tests:**
```bash
bash tests/scripts/test_get_set_balance.sh
bash tests/scripts/test_token_balance.sh
bash tests/scripts/test_isolation.sh
bash tests/scripts/test_mainnet_fork.sh
bash tests/scripts/test_send_transaction.sh
bash tests/scripts/test_spl_token.sh
bash tests/scripts/test_token_balance.sh 
bash tests/scripts/test_transaction_recording.sh
```

---

## API Documentation

### Fork Management

#### 1. Create Fork
```bash
curl -X POST http://localhost:3000/fork/create
```

**Response:**
```json
{
  "fork_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Each fork is isolated and expires after 15 minutes.

---

### Standard RPC Methods

#### 2. Get Balance
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getBalance",
    "params": ["<solana_address>"]
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": 5000000000
}
```

**Note:** If the account doesn't exist in the fork, it will be automatically fetched from mainnet.

---

#### 3. Get Account Info
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getAccountInfo",
    "params": ["<solana_address>"]
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "value": {
      "lamports": 1000000000,
      "owner": "11111111111111111111111111111111",
      "data": ["", "base58"],
      "executable": false,
      "rentEpoch": 0
    }
  }
}
```

---

#### 4. Get Latest Blockhash
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getLatestBlockhash",
    "params": []
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "blockhash": "CmpNeggWJ4JaWJeJ8YKN1Zypmk7uvQq3PECGUCAEMbky"
  }
}
```

---

#### 5. Send Transaction
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "sendTransaction",
    "params": ["<base58_encoded_signed_transaction>"]
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "5JK8z3xB9F2nP7wY..."
}
```

**See:** `tests/scripts/test_send_transaction.sh` for complete example with transaction creation.

---

### Cheatcode Methods

#### 6. Set Balance (SOL)
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_balance",
    "params": {
      "address": "<solana_address>",
      "lamports": 5000000000
    }
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "Success"
}
```

---

#### 7. Set Token Balance (SPL)
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_token_balance",
    "params": {
      "owner": "<wallet_address>",
      "mint": "<token_mint_address>",
      "amount": 1000000000
    }
  }'
```

**Example with USDC:**
```bash
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_token_balance",
    "params": {
      "owner": "D2bJqkFEa65xFKii3dW2ByrZEitdpX3PLR9uezPoSNKi",
      "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "amount": 1000000000
    }
  }'
```

**Note:** Amount is in smallest units (USDC has 6 decimals, so 1000000000 = 1000 USDC)

---

#### 8. Get Token Balance
```bash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "get_token_balance",
    "params": {
      "owner": "<wallet_address>",
      "mint": "<token_mint_address>"
    }
  }'
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": 1000000000
}
```

---

#### 9. Get Transaction History
```bash
curl http://localhost:3000/fork/{fork_id}/transactions
```

**Response:**
```json
{
  "transactions": [
    {
      "signature": "5JK8z3xB9F2nP7wY...",
      "timestamp": "2025-11-04T14:30:45+00:00",
      "success": true
    }
  ]
}
```

---

## Use Cases

### 1. Testing DeFi Protocols
```bash
# Create fork
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)

# Fund test wallet with 100 SOL
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_balance",
    "params": {
      "address": "YourWalletAddress",
      "lamports": 100000000000
    }
  }'

# Now safely test deposits, swaps, etc. without spending real SOL
```

### 2. Testing with Mainnet State
```bash
# Query real mainnet account - automatically fetched!
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getAccountInfo",
    "params": ["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]
  }'

# Modify it in your fork (mainnet unchanged)
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_balance",
    "params": {
      "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "lamports": 10000000000
    }
  }'
```

---

## Architecture
```
src/
├── main.rs              # HTTP server (Axum) and routing
├── fork_manager.rs      # Fork lifecycle, isolation, cleanup
└── rpc/
    ├── mod.rs          # RPC module exports
    ├── standard.rs     # Standard Solana RPC methods
    └── cheatcodes.rs   # Custom state manipulation methods
```

### Key Components

**ForkManager**
- Creates and manages fork lifecycle
- Handles automatic cleanup (15-minute TTL)
- Provides isolation between forks
- Auto-fetches missing accounts from mainnet

**Standard RPC Handler**
- Implements Solana-compatible RPC methods
- `getBalance`, `getAccountInfo`, `sendTransaction`, `getLatestBlockhash`,`get_token_balance`
- Compatible with existing Solana tools

**Cheatcodes Handler**
- Custom methods for testing
- `set_balance` - Instantly set SOL balance
- `set_token_balance` - Instantly set SPL token balance

---

## Implementation Details

### Fork Lifecycle

1. User requests fork creation via `POST /fork/create`
2. Server creates isolated liteSVM instance
3. Fork receives unique UUID
4. Fork automatically expires after 15 minutes
5. Cleanup task removes expired forks

### Fork Isolation

Each fork maintains completely separate:
- ✅ Account states (balances, data)
- ✅ Transaction history
- ✅ liteSVM instance
- ✅ On-demand loaded mainnet accounts

**Changes in one fork never affect other forks or mainnet.**

### Mainnet Account Fetching

When an account is accessed but not present in the fork:
1. Fork checks local state
2. If missing, fetches from Solana mainnet RPC
3. Caches account in fork
4. Returns account data

This enables testing with real mainnet state without pre-loading everything.

---

## Tech Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| VM | [liteSVM](https://github.com/LiteSVM/litesvm) 0.8 | Fast Solana VM simulation |
| Web Server | [Axum](https://github.com/tokio-rs/axum) 0.8 | HTTP routing and handlers |
| Async Runtime | [Tokio](https://tokio.rs/) 1.48 | Async I/O and task scheduling |
| RPC Client | solana-client 3.0 | Mainnet account fetching |
| Serialization | serde + serde_json | JSON-RPC protocol |

---

## Testing

### Unit Tests (14 tests)
```bash
cargo test -- --nocapture
```

Tests cover:
- Fork creation and isolation
- Balance operations (set/get)
- Multiple accounts per fork
- Invalid input handling
- Concurrent access
- SPL token operations
- Account info retrieval
- Transaction history

### Integration Tests (4 scripts)
```bash
# Test balance operations and fork isolation
bash tests/scripts/test_get_set_balance.sh

# Test forks for different users remain isolated
bash tests/scripts/test_isolation.sh

# Test mainnet info fetching 
bash tests/scripts/test_mainnet_fork.sh

# Test transaction execution
bash tests/scripts/test_send_transaction.sh

# Test spl token integration
bash tests/scripts/test_spl_token.sh

# Test SPL token balance manipulation
bash tests/scripts/test_token_balance.sh

# Test transaction recording
bash tests/scripts/test_transaction_recording.sh

```

---

## License

MIT License

---

## Author

Akshat (Parzival)

Built for Fluid Engineering Assignment

**GitHub**: https://github.com/parzival1821/solana-simulation-engine

---

## Acknowledgments

- [liteSVM](https://github.com/LiteSVM/litesvm) - Solana VM simulation
- [Solana Labs](https://github.com/solana-labs/solana) - Solana SDK
- Inspired by [Tenderly](https://tenderly.co) for Ethereum