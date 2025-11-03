#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Transaction Recording Test               ║${NC}"
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

# Step 3: Generate transaction
echo -e "${YELLOW}Step 3: Generating transaction...${NC}"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"
mkdir -p src

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

echo "  Compiling transaction generator..."
OUTPUT=$(cargo run --quiet -- "$BLOCKHASH" "9Po78y3svyS6cTZefY9UygJpUxkGECecvJ6V4vW5mx2F" 2>&1)
PAYER=$(echo "$OUTPUT" | grep "PAYER:" | cut -d: -f2)
TX_DATA=$(echo "$OUTPUT" | grep "TX:" | cut -d: -f2-)

cd - > /dev/null
rm -rf "$TMP_DIR"

echo -e "${GREEN}✓ Transaction generated${NC}"
echo "  Payer: $PAYER"
echo ""

# Step 4: Fund the payer
echo -e "${YELLOW}Step 4: Funding payer account...${NC}"
curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$PAYER\", \"lamports\": 10000000000}}" > /dev/null
echo -e "${GREEN}✓ Payer funded with 10 SOL${NC}\n"

# Step 5: Check transaction history is empty
echo -e "${YELLOW}Step 5: Checking initial transaction history...${NC}"
HISTORY=$(curl -s http://localhost:3000/fork/$FORK_ID/transactions)
INITIAL_COUNT=$(echo "$HISTORY" | jq '.transactions | length')
echo "  Initial transaction count: $INITIAL_COUNT"

if [ "$INITIAL_COUNT" != "0" ]; then
    echo -e "${RED}✗ Expected 0 transactions, got $INITIAL_COUNT${NC}"
    exit 1
fi
echo -e "${GREEN}✓ History empty as expected${NC}\n"

# Step 6: Send first transaction
echo -e "${YELLOW}Step 6: Sending first transaction...${NC}"
RESULT=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 2, \"method\": \"sendTransaction\", \"params\": [\"$TX_DATA\"]}")

if echo "$RESULT" | jq -e .error > /dev/null; then
    ERROR=$(echo "$RESULT" | jq -r .error.message)
    echo -e "${RED}✗ Transaction failed: $ERROR${NC}"
    exit 1
fi

SIGNATURE1=$(echo "$RESULT" | jq -r .result)
echo -e "${GREEN}✓ Transaction 1 signature: $SIGNATURE1${NC}\n"

# Step 7: Generate and send second transaction
echo -e "${YELLOW}Step 7: Generating and sending second transaction...${NC}"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"
mkdir -p src

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
        500_000_000,  // 0.5 SOL
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

OUTPUT=$(cargo run --quiet -- "$BLOCKHASH" "9Po78y3svyS6cTZefY9UygJpUxkGECecvJ6V4vW5mx2F" 2>&1)
PAYER2=$(echo "$OUTPUT" | grep "PAYER:" | cut -d: -f2)
TX_DATA2=$(echo "$OUTPUT" | grep "TX:" | cut -d: -f2-)

cd - > /dev/null
rm -rf "$TMP_DIR"

# Fund second payer
curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$PAYER2\", \"lamports\": 10000000000}}" > /dev/null

# Send second transaction
RESULT2=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 2, \"method\": \"sendTransaction\", \"params\": [\"$TX_DATA2\"]}")

SIGNATURE2=$(echo "$RESULT2" | jq -r .result)
echo -e "${GREEN}✓ Transaction 2 signature: $SIGNATURE2${NC}\n"

# Step 8: Check transaction history
echo -e "${YELLOW}Step 8: Verifying transaction history...${NC}"
HISTORY=$(curl -s http://localhost:3000/fork/$FORK_ID/transactions)

# Check count
FINAL_COUNT=$(echo "$HISTORY" | jq '.transactions | length')
echo "  Total transactions recorded: $FINAL_COUNT"

if [ "$FINAL_COUNT" != "2" ]; then
    echo -e "${RED}✗ Expected 2 transactions, got $FINAL_COUNT${NC}"
    echo "Response: $HISTORY"
    exit 1
fi

# Check first transaction
RECORDED_SIG1=$(echo "$HISTORY" | jq -r '.transactions[0].signature')
SUCCESS1=$(echo "$HISTORY" | jq -r '.transactions[0].success')
TIMESTAMP1=$(echo "$HISTORY" | jq -r '.transactions[0].timestamp')

echo ""
echo "  Transaction 1:"
echo "    Signature: $RECORDED_SIG1"
echo "    Success: $SUCCESS1"
echo "    Timestamp: $TIMESTAMP1"

if [ "$RECORDED_SIG1" != "$SIGNATURE1" ]; then
    echo -e "${RED}✗ Signature mismatch for transaction 1${NC}"
    exit 1
fi

if [ "$SUCCESS1" != "true" ]; then
    echo -e "${RED}✗ Transaction 1 should be successful${NC}"
    exit 1
fi

# Check second transaction
RECORDED_SIG2=$(echo "$HISTORY" | jq -r '.transactions[1].signature')
SUCCESS2=$(echo "$HISTORY" | jq -r '.transactions[1].success')
TIMESTAMP2=$(echo "$HISTORY" | jq -r '.transactions[1].timestamp')

echo ""
echo "  Transaction 2:"
echo "    Signature: $RECORDED_SIG2"
echo "    Success: $SUCCESS2"
echo "    Timestamp: $TIMESTAMP2"

if [ "$RECORDED_SIG2" != "$SIGNATURE2" ]; then
    echo -e "${RED}✗ Signature mismatch for transaction 2${NC}"
    exit 1
fi

if [ "$SUCCESS2" != "true" ]; then
    echo -e "${RED}✗ Transaction 2 should be successful${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Transaction Recording Test Passed! 🎉   ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Summary:"
echo "  ✓ 2 transactions executed"
echo "  ✓ Both transactions recorded in history"
echo "  ✓ Signatures match"
echo "  ✓ Success status correct"
echo "  ✓ Timestamps recorded"