Here’s your cleaned-up and fully Markdown-compliant README.md — all syntax, indentation, and code block formatting fixed:

⸻


# PBFT Proof of Concept

A lightweight **Byzantine Fault Tolerance (PBFT)** simulation written in **Rust**.  
This project demonstrates how nodes in a distributed system can communicate, broadcast, and handle consensus messages across a small cluster using **Axum**, **Reqwest**, and **tracing**.

---

## 🧠 Overview

Each node runs as an independent HTTP server (via **Axum**) that:

- Loads its identity and peer list from a shared `config/cluster.toml`.
- Exposes endpoints for health checks, peer discovery, and PBFT message handling.
- Logs all events to per-node files under the `logs/` directory.
- Demonstrates the PBFT flow with `Proposal`, `Prepare`, and `Commit` messages.

The simulation currently supports:

- ✅ Cluster of 4 nodes (1 potential Byzantine / bad node configurable later)
- ✅ Config-driven peer registry
- ✅ Typed PBFT message handling
- ✅ Broadcasting via HTTP
- ✅ Structured per-node file logging

---

## 🏗️ Project Structure

PBFT_POC/
├── Cargo.toml                # Dependencies and build metadata
├── config/
│   └── cluster.toml          # Cluster configuration (hosts and ports)
├── logs/                     # Log files created at runtime (per node)
├── scripts/
│   └── broadcast_proposal.sh # Helper script to test message broadcast
└── src/
├── main.rs               # Entry point – spawns all nodes
├── config_load.rs        # Loads cluster.toml config
├── logging.rs            # File-based tracing setup per node
├── node.rs               # Node startup & broadcast logic
├── routes.rs             # HTTP handlers
└── types.rs              # Shared PBFT message and response types

---

## ⚙️ Configuration

Edit `config/cluster.toml` to define your cluster:

```toml
[[nodes]]
id = 0
host = "127.0.0.1"
port = 8080

[[nodes]]
id = 1
host = "127.0.0.1"
port = 8081

[[nodes]]
id = 2
host = "127.0.0.1"
port = 8082

[[nodes]]
id = 3
host = "127.0.0.1"
port = 8083

Add or remove nodes here to simulate larger networks.

⸻

🚀 Run the Simulation

1. Build and start all nodes

RUST_LOG=info cargo run

This spawns 4 nodes locally:

127.0.0.1:8080
127.0.0.1:8081
127.0.0.1:8082
127.0.0.1:8083

2. Check cluster health

curl -s http://127.0.0.1:8080/peers | jq .

3. Send a PBFT Proposal

./scripts/broadcast_proposal.sh

Or manually:

curl -s -X POST http://127.0.0.1:8080/broadcast \
  -H 'content-type: application/json' \
  -d '{"kind":"Proposal","height":1,"proposer":0,"block":{"txs":[{"k":"k1","v":"v1"}]}}' | jq .

Expected Output
	•	Node 0 broadcasts a Proposal.
	•	Nodes 1–3 log recv Proposal.
	•	Each honest node rebroadcasts a Prepare.
	•	All nodes log incoming Prepare messages.

⸻

🧾 Logs

Each node writes logs under the logs/ directory:

logs/
├── node-0.log
├── node-1.log
├── node-2.log
└── node-3.log

Tail any log in real time:

tail -f logs/node-2.log


⸻

🔍 Next Steps (Planned)
	•	Add quorum tracking for Prepare and Commit phases (2f + 1 logic)
	•	Simulate one Byzantine (faulty) node
	•	Add cryptographic signatures or message digests
	•	Implement block execution and commit decision
	•	Add /metrics endpoint to expose PBFT state

⸻

🧰 Tech Stack

Component	Purpose
Rust + Tokio	Async runtime for concurrent nodes
Axum	Lightweight HTTP server for message endpoints
Reqwest	HTTP client for node-to-node broadcasting
tracing / tracing-subscriber / tracing-appender	Structured, per-node file logging
config	Loads cluster.toml for node topology
serde / serde_json	JSON serialization for messages

🤝 Author

Abhyudai Singh
📧 abhyudaisingh09@gmail.com
🔗 github.com/AbhyudaiSingh09
