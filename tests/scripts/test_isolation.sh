# Create two forks
FORK1=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)
FORK2=$(curl -s -X POST http://localhost:3000/fork/create | jq -r .fork_id)

# Set different balances
curl -X POST http://localhost:3000/fork/$FORK1/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "set_balance", "params": {"address": "11111111111111111111111111111111", "lamports": 1000000000}}'

curl -X POST http://localhost:3000/fork/$FORK2/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "set_balance", "params": {"address": "11111111111111111111111111111111", "lamports": 9000000000}}'

# Check they're different
curl -X POST http://localhost:3000/fork/$FORK1/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "getBalance", "params": ["11111111111111111111111111111111"]}'
# Should return 1000000000

curl -X POST http://localhost:3000/fork/$FORK2/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "getBalance", "params": ["11111111111111111111111111111111"]}'
# Should return 9000000000