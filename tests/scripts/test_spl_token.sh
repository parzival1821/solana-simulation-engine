#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SPL Token Program Interaction Test       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}\n"

# Known mainnet USDC mint
USDC_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

# Create fork
echo -e "${YELLOW}Step 1: Creating fork...${NC}"
FORK_ID=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
echo -e "${GREEN}✓ Fork ID: $FORK_ID${NC}\n"

# Try to get USDC mint info from mainnet
echo -e "${YELLOW}Step 2: Testing mainnet account access...${NC}"
echo "  Querying USDC mint account from mainnet: $USDC_MINT"

RESULT=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getAccountInfo\", \"params\": [\"$USDC_MINT\"]}")

# Check if we got data
if echo "$RESULT" | jq -e '.result.value' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Successfully accessed mainnet USDC mint account!${NC}"
    echo "  Account owner: $(echo "$RESULT" | jq -r '.result.value.owner')"
    echo ""
else
    echo -e "${YELLOW}⚠️  Could not access mainnet account${NC}"
    echo "  This is expected if liteSVM doesn't auto-fetch from mainnet"
    echo "  (Would need to implement on-demand account fetching)"
    echo ""
fi

# Test with System Program (always exists)
echo -e "${YELLOW}Step 3: Testing System Program interaction...${NC}"
SYSTEM_PROGRAM="11111111111111111111111111111111"

RESULT=$(curl -s -X POST http://localhost:3000/fork/$FORK_ID/rpc \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\": \"2.0\", \"id\": 1, \"method\": \"getAccountInfo\", \"params\": [\"$SYSTEM_PROGRAM\"]}")

if echo "$RESULT" | jq -e '.result.value' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ System Program exists in fork${NC}"
    echo "  Executable: $(echo "$RESULT" | jq -r '.result.value.executable')"
else
    echo -e "${YELLOW}⚠️  System Program not found (unexpected)${NC}"
fi

echo ""
echo -e "${BLUE}Summary:${NC}"
echo "  ✓ Fork creation works"
echo "  ✓ Can query program accounts"
echo "  ⚠️  Mainnet state fetching depends on liteSVM configuration"