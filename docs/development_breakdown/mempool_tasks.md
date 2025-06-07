# 🧊 Mempool: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 3: Mempool Module](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-3-mempool-module)

**Likely Crates/Tools:**
- `std::collections::{HashMap, VecDeque, BTreeSet}` (for internal data structures)
- `tokio::sync::{Mutex, RwLock}` (if accessed concurrently, common in node context)
- `tracing` (for logging mempool activity)

This document lists the actionable development tasks for implementing the RustChain Mempool.

The Mempool is responsible for temporarily storing and managing transactions that have been broadcast to the network but not yet included in a block.

## I. Module Definition and Core Structure

- [ ] **Define `mempool.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Design internal data structure for the mempool.** (DEVELOPMENT_FLOW.md Phase 3)
    - Considerations: Efficient addition, removal, deduplication, and retrieval of transactions.
    - Potential structures: `HashMap<TxHash, Transaction>` for quick lookups, `VecDeque<TxHash>` for ordering (e.g., FIFO or priority-based), `BTreeSet` for sorted transactions (e.g. by fee/nonce).
    - For a minimal version, a `HashMap` for deduplication and a `VecDeque` for ordering might suffice.
- [ ] Define `MempoolConfig` struct (e.g., max size, max tx per sender, min fee rate - for later enhancements).
- [ ] Define `Mempool` struct to encapsulate its data and logic.
- [ ] Implement `new(config: MempoolConfig) -> Self` constructor.

## II. Core Functionality

- [ ] **Implement transaction addition.** (DEVELOPMENT_FLOW.md Phase 3)
    - Function signature: `add_transaction(&mut self, transaction: Transaction) -> Result<(), MempoolError>`.
    - **Deduplication:** Ensure the same transaction (by hash) isn't added multiple times.
    - Basic validation (quick checks, defer full validation to state machine/consensus if not done before hitting mempool):
        - Transaction signature (if not already verified by network layer).
        - Transaction size limits.
        - (Future: fee checks, nonce checks against mempool state).
- [ ] **Implement transaction removal.** (DEVELOPMENT_FLOW.md Phase 3)
    - Function signature: `remove_transaction(&mut self, tx_hash: &Hash) -> Option<Transaction>`.
    - Needed when transactions are included in a block or become invalid.
- [ ] **Implement retrieval of transactions for block building.** (DEVELOPMENT_FLOW.md Phase 3)
    - Function signature: `get_transactions_for_block(&self, max_txs: usize, max_size_bytes: usize) -> Vec<Transaction>`.
    - Logic to select transactions (e.g., oldest, highest fee - for now, simple FIFO is fine).
- [ ] Implement a way to query the mempool status (e.g., number of transactions, total size).
    - Function signature: `get_status(&self) -> MempoolStatus` (define `MempoolStatus` struct).

## III. Error Handling

- [ ] Define `MempoolError` enum.
    - E.g., `TransactionExists`, `MempoolFull`, `InvalidTransaction` (for basic checks done by mempool).

## IV. Concurrency (Initial Consideration)

- [ ] Wrap internal data structures with appropriate locks (e.g., `tokio::sync::Mutex` or `RwLock`) if the mempool will be accessed from multiple asynchronous tasks (likely in a node environment).
    - This might be deferred slightly until node integration but good to keep in mind.

## V. Unit Tests

- [ ] Test adding a valid transaction.
- [ ] Test adding a duplicate transaction (should be ignored or error).
- [ ] Test retrieving transactions for block building (empty, with some, respects limits).
- [ ] Test removing a transaction.
- [ ] Test mempool capacity limits (if implemented early).
- [ ] ✅ **Milestone Check:** Transactions can be added to the mempool, deduplicated, and retrieved for block creation. (Corresponds to DEVELOPMENT_FLOW.md Phase 3 Milestone: "Transactions accepted into mempool and retrieved for block")

## VI. Logging

- [ ] Add `tracing` logs for key mempool operations (e.g., transaction added, removed, block candidate created). 