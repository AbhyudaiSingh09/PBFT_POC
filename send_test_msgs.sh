# Bash script: send_test_msgs.sh
# ./send_test_msgs.sh
#!/usr/bin/env bash
set -euo pipefail

# Simple test script to send a sample message to all 4 nodes

for port in 8080 8081 8082 8083; do
  echo "Sending test message to node at port $port..."
  curl -s -X POST http://127.0.0.1:${port}/msg \
    -H 'Content-Type: application/json' \
    -d '{"from":"client","kind":"Prepare","payload":{"height":1}}' \
    | jq .
  echo ""
done

echo "✅ Messages sent to all nodes."




# Run servers:
# $ RUST_LOG=info cargo run

# Acceptance check (from any node, e.g., node 0):
# $ curl -s -X POST http://127.0.0.1:8080/broadcast \
#   -H 'content-type: application/json' \
#   -d '{"from":"node-0","kind":"Hello","payload":{"height":1}}' | jq .

# Expected:
# - HTTP 200 with a JSON report: {"sender_id":0,"attempted":3,"succeeded":3}
# - Each other node (8081, 8082, 8083) logs a "received /msg" with that payload.


# Check peers:
$ curl -s http://127.0.0.1:8080/peers | jq .

# Broadcast a Proposal from node 0:
$ curl -s -X POST http://127.0.0.1:8080/broadcast \
  -H 'content-type: application/json' \
  -d '{"kind":"Proposal","height":1,"proposer":0,"block":{"txs":[{"k":"k1","v":"v1"}]}}' | jq .

# Expected:
# - node 1..3 logs "recv Proposal" and then each broadcasts a Prepare
# - node 0..3 logs incoming Prepare messages

