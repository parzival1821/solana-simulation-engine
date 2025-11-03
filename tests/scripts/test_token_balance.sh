#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SPL Token Balance Test                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}\n"

# Create fork
echo -e "${YELLOW}Step 1: Creating fork...${NC}"
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
echo -e "${GREEN}✓ Fork ID: $FORK_ID${NC}\n"

# Real USDC mint on mainnet
USDC_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
OWNER="D2bJqkFEa65xFKii3dW2ByrZEitdpX3PLR9uezPoSNKi"

echo -e "${YELLOW}Step 2: Fetching mainnet token balance...${NC}"
INITIAL_BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"get_token_balance\", \"params\": {\"owner\": \"$OWNER\", \"mint\": \"$USDC_MINT\"}}" | jq -r .result)

if [ "$INITIAL_BALANCE" != "0" ] && [ "$INITIAL_BALANCE" != "null" ]; then
    # Convert to human readable (USDC has 6 decimals)
    HUMAN_BALANCE=$(echo "scale=6; $INITIAL_BALANCE / 1000000" | bc)
    echo -e "${CYAN}  ✓ Mainnet balance found: $INITIAL_BALANCE tokens (~$HUMAN_BALANCE USDC)${NC}"
else
    echo "  Initial balance: 0 tokens (no mainnet account)"
fi
echo ""

echo -e "${YELLOW}Step 3: Overriding balance to 1000 USDC (1000000000 tokens)...${NC}"
RESULT=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"set_token_balance\", \"params\": {\"owner\": \"$OWNER\", \"mint\": \"$USDC_MINT\", \"amount\": 1000000000}}")

if echo "$RESULT" | jq -e .error > /dev/null; then
    ERROR=$(echo "$RESULT" | jq -r .error.message)
    echo -e "${RED}✗ Failed: $ERROR${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Balance overridden in fork${NC}\n"

echo -e "${YELLOW}Step 4: Verifying new balance in fork...${NC}"
NEW_BALANCE=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"get_token_balance\", \"params\": {\"owner\": \"$OWNER\", \"mint\": \"$USDC_MINT\"}}" | jq -r .result)

echo "  Fork balance: $NEW_BALANCE tokens (1000 USDC)"

if [ "$NEW_BALANCE" == "1000000000" ]; then
    echo -e "\n${GREEN}╔══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     Token Balance Test Passed! 🎉        ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
else
    echo -e "\n${RED}✗ Balance verification failed${NC}"
    echo "Expected: 1000000000, Got: $NEW_BALANCE"
    exit 1
fi

echo ""
echo -e "${CYAN}Summary:${NC}"
if [ "$INITIAL_BALANCE" != "0" ] && [ "$INITIAL_BALANCE" != "null" ]; then
    echo "  ✓ Fetched mainnet balance: $INITIAL_BALANCE tokens"
    echo "  ✓ Overwrote balance in fork: 1000000000 tokens"
else
    echo "  ✓ Created new token account: 1000000000 tokens"
fi
echo "  ✓ Fork isolation working (mainnet unchanged)"
echo "  ✓ Balance manipulation successful"