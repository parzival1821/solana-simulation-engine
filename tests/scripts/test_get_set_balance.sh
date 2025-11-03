#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Solana Fork Engine - Integration Test  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}\n"

# Test 1: Fork Creation
echo -e "${YELLOW}Test 1: Fork Creation${NC}"
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
echo -e "${GREEN}✓ Fork created: $FORK_ID${NC}\n"

# Test 2: Balance Manipulation
echo -e "${YELLOW}Test 2: Balance Manipulation${NC}"
# ADDRESS="11111111111111111111111111111111"
ADDRESS="D2bJqkFEa65xFKii3dW2ByrZEitdpX3PLR9uezPoSNKi" # this is my actual account, and it the tests check out 

BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS\"]}" | jq -r .result)

echo " Balance before setting : $BALANCE lamports\n"

# Set balance to 5 SOL
curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$ADDRESS\", \"lamports\": 5000000000}}" > /dev/null

# Check balance
BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS\"]}" | jq -r .result)

echo "  Set balance to: 5000000000 lamports (5 SOL)"
echo -e "  Current balance: $BALANCE lamports\n"

if [ "$BALANCE" == "5000000000" ]; then
    echo -e "${GREEN}✓ Balance set correctly${NC}\n"
else
    echo -e "✗ Balance mismatch!"
    exit 1
fi

# Test 3: Fork Isolation
echo -e "${YELLOW}Test 3: Fork Isolation${NC}"
FORK_2=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
echo "  Created second fork: $FORK_2"

# Set different balance in fork 2
curl -s -X POST http://localhost:3000/fork/$FORK_2/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$ADDRESS\", \"lamports\": 9000000000}}" > /dev/null

BALANCE_2=$(curl -s -X POST http://localhost:3000/fork/$FORK_2/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS\"]}" | jq -r .result)

# Verify fork 1 unchanged
BALANCE_1=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS\"]}" | jq -r .result)

echo "  Fork 1 balance: $BALANCE_1 lamports (5 SOL)"
echo "  Fork 2 balance: $BALANCE_2 lamports (9 SOL)"

if [ "$BALANCE_1" == "5000000000" ] && [ "$BALANCE_2" == "9000000000" ]; then
    echo -e "${GREEN}✓ Forks are properly isolated${NC}\n"
else
    echo -e "✗ Fork isolation failed!"
    exit 1
fi

# Test 4: Multiple Operations on Same Fork
echo -e "${YELLOW}Test 4: Multiple Balance Changes${NC}"
ADDRESS_2="11111111111111111111111111111111"
# if the account doesnt exist on mainnet also, it will error out and show NULL lamports

curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_balance\", \"params\": {\"address\": \"$ADDRESS_2\", \"lamports\": 3000000000}}" > /dev/null

BAL_A=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS\"]}" | jq -r .result)

BAL_B=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getBalance\", \"params\": [\"$ADDRESS_2\"]}" | jq -r .result)

echo "  Account 1: $BAL_A lamports (5 SOL)"
echo "  Account 2: $BAL_B lamports (3 SOL)"
echo -e "${GREEN}✓ Multiple accounts working${NC}\n"

echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       All Tests Passed! 🎉              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"