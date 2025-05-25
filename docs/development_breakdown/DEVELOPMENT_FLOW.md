# 🚧 RustChain Phase Plan

This roadmap outlines the major development phases of **RustChain**, a minimal Layer 1 Proof-of-Stake blockchain written in Rust. Each phase builds toward a fully working peer-to-peer blockchain system.

---

## 🧱 Phase 0: Project Scaffolding & Module Setup

- Create Rust project with `cargo new`
- Setup module layout:
src/
├── main.rs
├── lib.rs
├── block.rs
├── transaction.rs
├── mempool.rs
├── state_machine.rs
├── consensus.rs
├── storage.rs
├── networking.rs
├── validator.rs
└── types.rs

- Add placeholder `mod` statements in `lib.rs`
- Define common types in `types.rs`

✅ **Milestone**: Clean project layout with module boundaries in place

---

## 🔐 Phase 1: Key Management & CLI Wallet

- Generate Ed25519 or Secp256k1 keypairs
- Derive public address from public key
- Store/load private key from file (dev-mode only)
- Sign and serialize transactions
- CLI commands:
- `wallet generate`
- `wallet show`
- `wallet send`

✅ **Milestone**: First signed transaction created from CLI

---

## 🧾 Phase 2: Transaction Structs & Validation Logic

- Define `Transaction` struct
- Implement:
- `hash_transaction()`
- `verify_signature()`
- `validate_transaction()` (nonce, balance, sig)
- Define `TxValidationError` enum
- Unit test for transaction validity

✅ **Milestone**: First transaction validated with test state

---

## 🧊 Phase 3: Mempool Module

- In-memory transaction queue
- Add, deduplicate, remove transactions
- Provide top N transactions for block builder

✅ **Milestone**: Transactions accepted into mempool and retrieved for block

---

## 🧱 Phase 4: Block Structure & Merkle Tree

- Define `BlockHeader` and `Block`
- Calculate Merkle root over transactions
- Hash and sign block headers
- Serialize block

✅ **Milestone**: Signed block with valid Merkle root generated

---

## ⚖️ Phase 5: Consensus Engine (Static PoS)

- Define validator list in genesis
- Implement round-robin proposer logic
- Validate proposer per block
- Longest-chain fork rule

✅ **Milestone**: Validator node correctly produces and accepts blocks

---

## 🔁 Phase 6: State Machine Execution

- Define `Account`, `State`
- Apply transactions and update state
- Validate balances, nonce
- Create recipient accounts if needed

✅ **Milestone**: Balances updated based on txs in block

---

## 🗃️ Phase 7: Storage Layer (RocksDB or Sled)

- Store blocks, headers, state, and metadata:
blocks/{hash}
headers/{height}
state/{address}
meta/tip

- Load chain tip and account state on restart

✅ **Milestone**: Node restarts and recovers full state

---

## 🌐 Phase 8: Networking Layer (libp2p)

- Setup Swarm, Gossipsub, Noise, Yamux
- Support:
- `TxMessage`
- `BlockMessage`
- `SyncRequest`, `SyncResponse`
- Broadcast txs and blocks
- Deduplicate messages by hash

✅ **Milestone**: Transaction sent by Node A is received by Node B

---

## 🧩 Phase 9: Block Production Integration

- Validator:
- Collects txs from mempool
- Builds and signs block
- Broadcasts block to peers
- Peers validate, apply, and persist block

✅ **Milestone**: Nodes successfully produce and accept blocks

---

## 🔗 Phase 10: Basic Chain Sync

- Sync from peers on startup
- Request headers/blocks from known tip
- Catch up to latest height

✅ **Milestone**: New node syncs chain to latest block from peers

---

## 🧪 Phase 11: Genesis File & Dev Fixtures

- Define `genesis.json`:
- Validators, balances, initial nonce
- Load state from genesis
- Add test transactions to dev folder
- CLI support: `--genesis path/to/file`

✅ **Milestone**: Node loads state and validators from JSON genesis

---

## ⚙️ Phase 12: Configuration & CLI Flags

- Load:
- Port, database path
- Validator key
- Peers
- From `config.toml` or CLI args

✅ **Milestone**: Node is configurable via file or CLI

---

## 🛠️ Phase 13: Manual Testing & Hardening

- Manual test invalid txs and fork behavior
- Improve logs and error messages
- Test edge cases:
- Double spend
- Invalid proposer
- Invalid signature

✅ **Milestone**: Local testnet reliably handles failure cases

---

## 🧪 Phase 14: Unit & Integration Tests

- Add unit tests for:
- Transaction logic
- Block hashing
- Merkle root
- Add integration tests for:
- Syncing
- End-to-end wallet → tx → block flow

✅ **Milestone**: Full stack tests pass for consensus and state transitions

---

## 🕵️ Phase 15: Optional Block Explorer (CLI)

- CLI tool:
- Query block by height/hash
- Query account balance
- Query transaction by hash
- Read from RocksDB

✅ **Milestone**: Explorer inspects on-disk blockchain data

---

## 📦 Final Phase: Polish & Publish

- Record demo video
- Write detailed `README.md`
- List features and usage instructions
- Push to GitHub

✅ **Milestone**: Project is showcase-ready 