# Solana Fork Engine

A lightweight Solana simulation engine that creates isolated network forks for testing dapp interactions without touching mainnet.

## Features

- ✅ **Isolated Forks**: Each user gets their own fork of Solana state
- ✅ **Balance Manipulation**: Set SOL balances instantly via cheatcodes
- ✅ **Standard RPC**: Compatible with standard Solana RPC methods
- ✅ **Auto Cleanup**: Forks automatically expire after 15 minutes
- ✅ **Fast & Lightweight**: Built on liteSVM for in-memory simulation

## Quick Start

### Installation
```bash
git clone <your-repo>
cd solana-fork-engine
cargo build --release
```

### Run Server
```bash
cargo run
# Server starts on http://localhost:3000
```

### Run Tests
```bash
./test.sh
```

## API Documentation

### 1. Create Fork
```bash
curl -X POST http://localhost:3000/fork/create
```

**Response:**
```json
{
  "fork_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 2. Get Balance (Standard RPC)
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

### 3. Set Balance (Cheatcode)
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

### 4. Send Transaction
```bash
# First get the latest blockhash
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "getLatestBlockhash", "params": []}'

# Create a properly signed transaction using the blockhash
# Then send it:
curl -X POST http://localhost:3000/fork/{fork_id}/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "sendTransaction",
    "params": ["<base58_encoded_transaction>"]
  }'
```

See `tests/scripts/test_send_transaction.sh` for a complete example.
```

### 🪙 SPL Token Support

Set and query SPL token balances:

#### Set Token Balance
```bash
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_token_balance",
    "params": {
      "owner": "<wallet_address>",
      "mint": "<token_mint>",
      "amount": 1000000000
    }
  }'
```

#### Get Token Balance
```bash
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "get_token_balance",
    "params": {
      "owner": "<wallet_address>",
      "mint": "<token_mint>"
    }
  }'
```

**Example with USDC:**
- Mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- Amount: `1000000000` (1000 USDC with 6 decimals)

## Advanced Features

### 🌐 Mainnet Account Fetching

The engine automatically fetches missing accounts from Solana mainnet:
```bash
# Query a real mainnet account - automatically fetched!
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "getBalance", 
       "params": ["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"]}'
```

### 🔮 Transaction Simulation

Preview transaction effects without executing:
```bash
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "simulateTransaction", 
       "params": [""]}'
```

## Use Cases

### Testing DeFi Protocols
```bash
# 1. Create fork
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)

# 2. Fund test wallet
curl -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "set_balance",
    "params": {
      "address": "YourWalletAddress",
      "lamports": 10000000000
    }
  }'

# 3. Test your protocol interactions
# (simulate deposits, swaps, etc.)
```


## Architecture
```
src/
├── main.rs              # HTTP server and routing
├── fork_manager.rs      # Fork lifecycle management
└── rpc/
    ├── mod.rs
    ├── standard.rs      # Standard RPC methods (getBalance)
    └── cheatcodes.rs    # Custom methods (set_balance)
```

### Key Components

- **ForkManager**: Manages fork lifecycle, isolation, and cleanup
- **Standard RPC**: Implements Solana-compatible RPC methods
- **Cheatcodes**: Custom methods for state manipulation

## Tech Stack

- [liteSVM](https://github.com/LiteSVM/litesvm) - Fast Solana VM simulation
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime

## Implementation Details

### Fork Lifecycle

1. User requests fork creation
2. Server creates isolated liteSVM instance
3. Fork receives unique ID
4. Fork expires after 15 minutes (automatic cleanup)

### Fork Isolation

Each fork maintains separate:
- Account states
- Balances
- Transaction history

Changes in one fork don't affect others.

## Limitations

- Forks are in-memory only (not persisted)
- 15-minute lifetime per fork
- Standard Solana RPC methods only (subset implemented)

## Future Enhancements

- [ ] SPL Token support
- [ ] Transaction simulation
- [ ] Extended RPC method support
- [ ] Fork persistence option

## License

MIT

## Author

Built for Fluid Engineering Assignment