# 🏛️ Validator Node Responsibilities

A validator node bundles all major subsystems:

- P2P Networking (libp2p)
- Mempool
- Consensus Engine
- State Manager
- Block Producer
- Chain DB handler

## 📡 Exposed Interfaces

- **P2P Network Interface:** For communication with other nodes (block/transaction propagation, sync).
- **JSON-RPC API:** For wallet interactions and external queries. Example endpoints:
    - `get_balance { "address": "0x..." }`
    - `get_nonce { "address": "0x..." }`
    - `submit_transaction { "tx_hex": "..." }`

## 🧵 Event Loop

Runs as a `tokio`-based async loop:

- Handles:
    - Incoming transactions from the P2P network and potentially from the RPC interface.
    - Block proposal initiation when triggered by the Consensus Engine (i.e., when it's this node's turn to propose).
    - Incoming blocks from peers via the P2P network.
    - Chain sync requests
    - Mempool cleanup

## 🔐 Validator Private Key

- Used to sign proposed blocks
- Stored securely (in dev: plaintext, in prod: encrypted)