# 🏛️ Validator Node: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 9: Block Production Integration](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-9-block-production-integration)
- [Phase 11: Genesis File & Dev Fixtures](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-11-genesis-file--dev-fixtures)
- [Phase 12: Configuration & CLI Flags](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-12-configuration--cli-flags)
- Integration with all other component tasks.

**Likely Crates/Tools:**
- `tokio` (async runtime, channels, tasks)
- `clap` (CLI argument parsing)
- `serde` & `serde_json` / `toml` (for config and genesis file parsing)
- `tracing` & `tracing-subscriber` (for structured logging)
- All other component crates (`libp2p`, `rocksdb`, `bincode`, etc.)

This document lists tasks for implementing the RustChain validator node, including setup, component integration, and core blockchain operations.

## I. Project & Node Scaffolding (Phase 0)

- [ ] **Create Rust project with `cargo new rustchain --bin`.** (DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Setup `main.rs` as the entry point for the node.** (DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Define `validator.rs` (or `node.rs`) module for main node logic.** (DEVELOPMENT_FLOW.md Phase 0)
- [ ] Setup `lib.rs` if library components are factored out; ensure `mod` statements connect all modules.

## II. Configuration and Genesis (Phases 11, 12)

- [ ] **Define `NodeConfig` struct.** (DEVELOPMENT_FLOW.md Phase 12)
    - Fields: P2P listen address/port, DB path, validator private key path (if applicable), bootstrap peer list, genesis file path.
- [ ] **Implement CLI argument parsing for config options.** (DEVELOPMENT_FLOW.md Phase 12)
    - Crate: `clap`.
    - E.g., `--config <path>`, `--port <port>`, `--db-path <path>`, `--genesis <path>`.
- [ ] Implement loading config from a TOML file (e.g., `config.toml`). (DEVELOPMENT_FLOW.md Phase 12)
    - Override with CLI args.
- [ ] **Define `GenesisData` struct.** (DEVELOPMENT_FLOW.md Phase 11)
    - Fields: Initial validator set (public keys), initial account balances and nonces, genesis timestamp.
    - Format: JSON (e.g., `genesis.json`).
- [ ] **Implement loading genesis data from JSON file.** (DEVELOPMENT_FLOW.md Phase 11)
    - Crate: `serde_json`.
- [ ] ✅ **Milestone Check (Config):** Node can be configured via a file and/or CLI arguments. (Corresponds to DEVELOPMENT_FLOW.md Phase 12 Milestone).

## III. Node Initialization & Component Setup

- [ ] **Implement node identity (PeerId) setup.**
    - Load or generate libp2p keypair (as defined in `p2p_networking_tasks.md`).
- [ ] **Initialize all core components based on config and genesis data:**
    - Storage Layer: Open/initialize database at `db_path`.
    - State Machine: Initialize with genesis state (balances, nonces from `GenesisData`).
        - Store initial genesis state to DB if not already present.
    - Consensus Engine: Initialize with static validator set from `GenesisData`.
    - Mempool: Initialize (e.g., empty, with config).
    - Networking Service: Initialize with PeerId, listen address, bootstrap nodes.
- [ ] Check if blockchain has been initialized before (e.g., by checking for genesis block in DB). If not, write genesis block and initial state derived from `GenesisData` to storage.
- [ ] ✅ **Milestone Check (Genesis):** Node correctly loads initial state (accounts, validators) from a genesis file. (Corresponds to DEVELOPMENT_FLOW.md Phase 11 Milestone).

## IV. Main Event Loop & Orchestration

- [ ] **Implement main async event loop using `tokio`.**
    - Use channels (`tokio::sync::mpsc`) for communication between components (e.g., Network -> Mempool, Network -> BlockProcessor, BlockProposer -> Network).
- [ ] **Handle incoming network messages:**
    - `TxMessage`: Pass to Mempool for validation and addition.
    - `BlockMessage`: Pass to Block Processor for validation and application.
    - `SyncRequest/Response`: Handle by sync coordinator.
- [ ] Implement graceful shutdown logic (Ctrl-C handler):
    - Signal all components to stop.
    - Ensure state is persisted by Storage Layer.
    - Close network connections cleanly.

## V. Block Production (Validator Role - Phase 9)

- [ ] **Implement block proposal timing logic (for current validator).**
    - Check with Consensus Engine if it's this node's turn to propose (`can_propose_block`).
    - Could be time-based (slots) or event-driven.
- [ ] **If it's time to propose:**
    - **Collect transactions from Mempool.** (DEVELOPMENT_FLOW.md Phase 9)
        - Adhere to block size/gas limits.
    - **Build a new block.** (DEVELOPMENT_FLOW.md Phase 9)
        - Create `BlockHeader` (parent hash from current tip, proposer, timestamp, Merkle root from collected transactions).
        - Sign the block header hash with the validator's private key.
        - Assemble `Block` struct.
    - **Apply the new block to local State Machine and Storage (optimistic application, may need rollback if rejected by network).**
    - **Broadcast the new block to peers via Networking Layer.** (DEVELOPMENT_FLOW.md Phase 9)
    - Log block production details.

## VI. Block Processing (All Nodes - Phase 9)

- [ ] **Implement a Block Processing pipeline for received blocks.**
- [ ] **Validate the received block:** (DEVELOPMENT_FLOW.md Phase 9)
    - Basic structural validation (Core Data Structures).
    - Consensus validation (proposer, signature, timestamp - Consensus Engine).
    - Stateful validation of all transactions within the block against current state (State Machine).
- [ ] **If block is valid:**
    - **Apply transactions to State Machine, updating world state.** (DEVELOPMENT_FLOW.md Phase 9)
    - **Persist the block and new state to Storage Layer (atomically).** (DEVELOPMENT_FLOW.md Phase 9)
    - Update chain tip.
    - Log block acceptance.
- [ ] If block is invalid, discard and log reason.
- [ ] Handle potential forks using the fork choice rule from Consensus Engine.
- [ ] ✅ **Milestone Check (Block Production/Acceptance):** Nodes can successfully produce new blocks (if validators) and receive, validate, apply, and persist blocks from peers. The chain progresses. (Corresponds to DEVELOPMENT_FLOW.md Phase 9 Milestone).

## VII. Chain Synchronization (Phase 10)

- [ ] **Implement initial chain synchronization logic on node startup.** (DEVELOPMENT_FLOW.md Phase 10)
    - If local chain is behind known peers (or from a checkpoint if implemented).
- [ ] **Design sync strategy:** (e.g., header-first sync then download block bodies).
    - Request headers from peers starting from local chain tip (or genesis if empty).
    - Validate received headers (using Consensus Engine).
    - Identify the best chain (longest valid chain) among peers.
- [ ] **Implement requesting block headers from peers.** (DEVELOPMENT_FLOW.md Phase 10)
    - Use the request-response protocol defined in `p2p_networking_tasks.md` (e.g., `GetHeadersRequest`).
    - Manage requests to multiple peers if necessary.
- [ ] **Implement requesting block bodies for validated headers.** (DEVELOPMENT_FLOW.md Phase 10)
    - Use the request-response protocol (e.g., `GetBlocksRequest`).
- [ ] **Process and persist synced blocks:**
    - Validate full blocks (Consensus & State Machine).
    - Apply to State Machine.
    - Store in Storage Layer.
    - Update chain tip.
- [ ] **Manage sync state:** Track current sync progress, target height, peers being synced from.
- [ ] Handle sync errors, peer disconnections during sync, and retries.
- [ ] Determine when initial sync is complete (e.g., caught up to network height or no new blocks from peers for a while).
    - **"Catch up to latest height."** (DEVELOPMENT_FLOW.md Phase 10)
- [ ] ✅ **Milestone Check (Chain Sync):** A new node, when started, can discover peers and synchronize the blockchain up to the latest block height maintained by the network. (Corresponds to DEVELOPMENT_FLOW.md Phase 10 Milestone).

## VIII. Logging and Monitoring

- [ ] Setup structured logging using `tracing` and `tracing-subscriber`.
- [ ] Add detailed logs for node startup, shutdown, component initialization, errors, block production, block acceptance/rejection, transaction processing.

## IX. Integration Tests

- [ ] Test node startup with a genesis file and config.
- [ ] Test a single validator node producing a series of blocks (with or without transactions).
- [ ] Test multiple nodes: one validator, one+ listeners. Ensure listeners sync blocks.
- [ ] Test transaction flow: CLI Wallet -> Node A (Validator) -> Mempool -> Block -> Network -> Node B -> Validation -> State Update. 