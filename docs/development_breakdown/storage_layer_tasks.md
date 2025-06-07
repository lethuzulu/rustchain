# 🗃️ Storage Layer: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 7: Storage Layer (RocksDB or Sled)](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-7-storage-layer--rocksdb-or-sled)

**Likely Crates/Tools:**
- `rocksdb` (primary choice, as per user's flow) or `sled`
- `bincode` & `serde` (for serializing/deserializing data before storage)
- `tracing` (for logging storage operations and errors)

This document lists the actionable development tasks for implementing RustChain's persistent storage layer.

## I. Module Definition and Backend Setup

- [ ] **Define `storage.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Choose and integrate a key-value store backend.** (DEVELOPMENT_FLOW.md Phase 7 indicates RocksDB or Sled).
    - For initial implementation, focus on one (e.g., RocksDB).
    - Crate: `rocksdb` (or `sled`).
    - Basic setup: Opening/closing the database, path configuration.
- [ ] Define `Storage` struct to encapsulate database interactions.
    - It will hold a reference to the DB instance.
- [ ] Implement `new(path: &Path) -> Result<Self, StorageError>` constructor.
- [ ] Define `StorageError` enum (e.g., `NotFound`, `Corruption`, `IoError`, `SerializationError`).

## II. Data Schema and Key Design

(Corresponds to DEVELOPMENT_FLOW.md Phase 7 schema ideas)
- [ ] **Define key prefixes or column families for different data types.**
    - E.g., `b"blocks_"`, `b"headers_"`, `b"state_"`, `b"meta_"`.
    - Or use RocksDB Column Families for better separation.
- [ ] **Key Design:**
    - Blocks: `blocks/{block_hash}` -> `SerializedBlock`
    - Headers: `headers/{block_height}` -> `SerializedBlockHeader` (Consider also `headers_hash/{block_hash}` -> `SerializedBlockHeader` if lookup by hash is frequent)
    - Account State: `state/{address}` -> `SerializedAccount`
    - Metadata: `meta/chain_tip` -> `BlockHash` (of the current chain tip)
    - Metadata: `meta/genesis_hash` -> `BlockHash`

## III. Core Storage Operations

- [ ] **Implement Block Storage & Retrieval:** (DEVELOPMENT_FLOW.md Phase 7)
    - `put_block(block: &Block) -> Result<(), StorageError>` (stores block by its hash).
    - `get_block(block_hash: &Hash) -> Result<Option<Block>, StorageError>`.
- [ ] **Implement Block Header Storage & Retrieval:** (DEVELOPMENT_FLOW.md Phase 7)
    - `put_header(header: &BlockHeader) -> Result<(), StorageError>` (stores header by its height and/or hash).
    - `get_header_by_height(height: BlockHeight) -> Result<Option<BlockHeader>, StorageError>`.
    - `get_header_by_hash(block_hash: &Hash) -> Result<Option<BlockHeader>, StorageError>`.
- [ ] **Implement Account State Storage & Retrieval:** (DEVELOPMENT_FLOW.md Phase 7)
    - `put_account(address: &Address, account: &Account) -> Result<(), StorageError>`.
    - `get_account(address: &Address) -> Result<Option<Account>, StorageError>`.
    - `put_world_state(state: &WorldState) -> Result<(), StorageError>` (iterates and puts accounts).
- [ ] **Implement Metadata Storage & Retrieval:** (DEVELOPMENT_FLOW.md Phase 7)
    - `put_chain_tip(block_hash: &Hash) -> Result<(), StorageError>`.
    - `get_chain_tip() -> Result<Option<Hash>, StorageError>`.
    - `put_genesis_hash(block_hash: &Hash) -> Result<(), StorageError>`.
    - `get_genesis_hash() -> Result<Option<Hash>, StorageError>`.

## IV. Atomic Writes & Consistency

- [ ] Implement atomic commit for block application (i.e., block data, header, state changes, and new tip are written together or not at all).
    - Using RocksDB `WriteBatch` or equivalent for other backends.
    - Function: `commit_block_atomic(block: &Block, new_state_changes: &WorldStateDelta, new_tip_hash: &Hash) -> Result<(), StorageError>` (where `WorldStateDelta` represents changes to accounts).

## V. Initialization and Recovery

- [ ] **Implement logic to load chain tip and account state on restart.** (DEVELOPMENT_FLOW.md Phase 7)
    - This is primarily achieved by `get_chain_tip()` and then allowing the State Machine to load necessary state via `get_account()` or a bulk load if designed.
    - The Storage layer itself mostly provides the primitives for this.

## VI. Unit / Integration Tests

- [ ] Test basic Put/Get operations for each data type (blocks, headers, accounts, metadata).
- [ ] Test storing and retrieving the chain tip.
- [ ] Test overwriting existing data (e.g., updating an account, chain tip).
- [ ] Test reading non-existent data (should return `Ok(None)` or specific error).
- [ ] Test atomic commit: ensure all data is written or none is in case of an error during the batch (harder to test, may need fault injection or careful setup).
- [ ] ✅ **Milestone Check:** The storage layer can persist all necessary blockchain data (blocks, state, metadata) and recover the latest chain tip upon restart. (Partially covers DEVELOPMENT_FLOW.md Phase 7 Milestone: "Node restarts and recovers full state" - full recovery also involves state machine and node logic).

## VII. Logging

- [ ] Add `tracing` logs for key storage operations: DB open/close, writing block, reading tip, errors. 