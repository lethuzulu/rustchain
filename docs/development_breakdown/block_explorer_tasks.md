# 🕵️ Block Explorer (CLI): Task Breakdown

**Owner:** TBD
**Status:** To Do (Optional Component)

**Relevant Development Flow Phases:**
- [Phase 15: Optional Block Explorer (CLI)](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-15-optional-block-explorer--cli)

**Likely Crates/Tools:**
- `clap` (CLI argument parsing)
- `rocksdb` (for direct read-only access to the database)
- `bincode` & `serde` (for deserializing data read from DB)
- `hex` (for displaying hashes, addresses)
- `chrono` (for formatting timestamps)

This document lists actionable tasks for implementing the optional RustChain CLI block explorer.
The explorer will provide read-only access to blockchain data stored by a RustChain node.

## I. Setup and CLI Structure

- [ ] **Create a new Rust binary project for the explorer (e.g., `cargo new rustchain-explorer --bin`).**
    - This keeps it separate from the main node executable.
- [ ] **Implement CLI argument parsing using `clap`.**
    - Main command: `rustchain-explorer`.
    - Subcommands: `block`, `transaction`, `account`.
    - Global options: `--db-path <path_to_node_db>` (required).

## II. Database Interaction (Read-Only)

- [ ] **Implement read-only access to the RocksDB database.** (DEVELOPMENT_FLOW.md Phase 15)
    - Re-use or adapt storage logic/structs (e.g., key schemas, deserialization for `Block`, `BlockHeader`, `Transaction`, `Account`).
    - Ensure the explorer opens the DB in read-only mode if possible, or is careful not to write.
- [ ] Define utility functions to fetch and deserialize specific data types (blocks by hash/height, transactions by hash, accounts by address).

## III. CLI Commands Implementation

### A. Block Queries
- [ ] **Implement `block <ID>` subcommand.**
    - **Query block by height/hash.** (DEVELOPMENT_FLOW.md Phase 15)
        - `rustchain-explorer --db-path <...> block <BLOCK_HEIGHT_OR_HASH>`
    - Display block details: header fields (height, hash, parent hash, timestamp, proposer, Merkle root, signature), list of transaction hashes in the block.
    - (Optional) `block latest [N]` to show the latest N blocks.

### B. Transaction Queries
- [ ] **Implement `transaction <TX_HASH>` subcommand.**
    - **Query transaction by hash.** (DEVELOPMENT_FLOW.md Phase 15)
        - `rustchain-explorer --db-path <...> transaction <TRANSACTION_HASH>`
    - Display transaction details: hash, sender, recipient, amount, nonce, signature, timestamp.
    - Indicate which block it was included in (if easily retrievable, e.g., if tx index is built).

### C. Account Queries
- [ ] **Implement `account <ADDRESS>` subcommand.**
    - **Query account balance.** (DEVELOPMENT_FLOW.md Phase 15)
        - `rustchain-explorer --db-path <...> account <ACCOUNT_ADDRESS>`
    - Display account details: address, balance, nonce.

## IV. Output Formatting

- [ ] Implement clear and human-readable output for all query results.
    - Use `hex` for hashes, addresses.
    - Format timestamps (e.g., using `chrono`).
    - Pretty print structs/lists.

## V. Error Handling

- [ ] Handle errors gracefully: DB connection issues, data not found, deserialization errors.
- [ ] Provide informative error messages to the user.

## VI. Testing (Manual / Basic)

- [ ] Manually test all CLI commands against a populated node database.
    - Query existing blocks, transactions, accounts.
    - Query non-existent data.
- [ ] ✅ **Milestone Check:** The CLI block explorer can successfully connect to a node's database and display details for blocks, transactions, and accounts as specified. (Corresponds to DEVELOPMENT_FLOW.md Phase 15 Milestone: "Explorer inspects on-disk blockchain data")

## VII. Documentation

- [ ] Add a section to the main `README.md` (or a separate `EXPLORER_README.md`) explaining how to build and use the block explorer CLI. 