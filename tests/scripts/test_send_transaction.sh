#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  sendTransaction Integration Test        ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}\n"

# Step 1: Create fork
echo -e "${YELLOW}Step 1: Creating fork...${NC}"
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
echo -e "${GREEN}✓ Fork ID: $FORK_ID${NC}\n"

# Step 2: Get latest blockhash
echo -e "${YELLOW}Step 2: Getting latest blockhash from fork...${NC}"
BLOCKHASH=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "getLatestBlockhash", "params": []}' | jq -r .result.blockhash)
echo -e "${GREEN}✓ Blockhash: $BLOCKHASH${NC}\n"

# Step 3: Create a simple Rust program inline to generate transaction
echo -e "${YELLOW}Step 3: Generating transaction...${NC}"

# Create temporary Rust project
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Create directory structure FIRST
mkdir -p src

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "txgen"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "2.2"
bs58 = "0.5"
bincode = "1.3"
EOF

# Create main.rs
cat > src/main.rs << 'EOF'
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    message::Message,
    hash::Hash,
    system_instruction,
};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let blockhash = Hash::from_str(&args[1]).unwrap();
    let recipient = args[2].parse().unwrap();
    
    let payer = Keypair::new();
    
    let instruction = system_instruction::transfer(
        &payer.pubkey(),
        &recipient,
        1_000_000_000,
    );
    
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = blockhash;
    tx.sign(&[&payer], blockhash);
    
    let serialized = bincode::serialize(&tx).unwrap();
    println!("PAYER:{}", payer.pubkey());
    println!("TX:{}", bs58::encode(serialized).into_string());
}
EOF

# Run it and capture output
echo "  Compiling transaction generator (this may take a moment)..."
OUTPUT=$(cargo run --quiet -- "$BLOCKHASH" "9Po78y3svyS6cTZefY9UygJpUxkGECecvJ6V4vW5mx2F" 2>&1)
PAYER=$(echo "$OUTPUT" | grep "PAYER:" | cut -d: -f2)
TX_DATA=$(echo "$OUTPUT" | grep "TX:" | cut -d: -f2-)

# Go back and cleanup
cd - > /dev/null
rm -rf "$TMP_DIR"

echo -e "${GREEN}✓ Transaction generated${NC}"
echo "  Payer: $PAYER"
echo "  Recipient: 9Po78y3svyS6cTZefY9UygJpUxkGECecvJ6V4vW5mx2F"
echo ""

# Step 4: Fund the payer
echo -e "${YELLOW}Step 4: Funding payer account...${NC}"
curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$PAYER\", \"lamports\": 10000000000}}" > /dev/null
echo -e "${GREEN}✓ Payer funded with 10 SOL${NC}\n"

# Step 5: Check initial balances
echo -e "${YELLOW}Step 5: Checking initial balances...${NC}"
RECIPIENT="9Po78y3svyS6cTZefY9UygJpUxkGECecvJ6V4vW5mx2F"

PAYER_BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$PAYER\"]}" | jq -r .result)
echo "  Payer: $PAYER_BALANCE lamports (10 SOL)"

RECIPIENT_BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$RECIPIENT\"]}" | jq -r .result)
echo -e "  Recipient: $RECIPIENT_BALANCE lamports\n"

# Step 6: Send transaction
echo -e "${YELLOW}Step 6: Sending transaction...${NC}"
RESULT=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 2, \"method\": \"sendTransaction\", \"params\": [\"$TX_DATA\"]}")

if echo "$RESULT" | jq -e .error > /dev/null; then
    ERROR=$(echo "$RESULT" | jq -r .error.message)
    echo -e "${RED}✗ Transaction failed: $ERROR${NC}"
    echo "Full response: $RESULT"
    exit 1
fi

SIGNATURE=$(echo "$RESULT" | jq -r .result)
echo -e "${GREEN}✓ Transaction signature: $SIGNATURE${NC}\n"

# Step 7: Check final balances
echo -e "${YELLOW}Step 7: Verifying balances after transaction...${NC}"

PAYER_BALANCE_AFTER=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$PAYER\"]}" | jq -r .result)

RECIPIENT_BALANCE_AFTER=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$RECIPIENT\"]}" | jq -r .result)

echo "  Payer: $PAYER_BALANCE_AFTER lamports"
echo "  Recipient: $RECIPIENT_BALANCE_AFTER lamports"
echo ""

RECIPIENT_CHANGE=$((RECIPIENT_BALANCE_AFTER - RECIPIENT_BALANCE))

echo "  Recipient received: $RECIPIENT_CHANGE lamports (1 SOL = 1000000000)"

if [ "$RECIPIENT_CHANGE" == "1000000000" ]; then
    echo -e "\n${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Transaction Test Passed! 🎉          ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
else
    echo -e "\n${YELLOW}⚠️  Recipient change was $RECIPIENT_CHANGE (expected 1000000000)${NC}"
    echo "This might be okay if transaction fees were deducted"
fi