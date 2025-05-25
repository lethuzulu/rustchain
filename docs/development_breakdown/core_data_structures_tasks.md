# 📦 Core Data Structures: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 2: Transaction Structs & Validation Logic](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-2-transaction-structs--validation-logic)
- [Phase 4: Block Structure & Merkle Tree](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-4-block-structure--merkle-tree)

**Likely Crates/Tools:**
- `bincode` (serialization)
- `serde` (Serialize, Deserialize traits)
- `sha2` (SHA-256 hashing)
- `merkle_cbt` or custom implementation for Merkle trees
- `ed25519-dalek` or `secp256k1` (for signature types, public key types, if defined here)

This document lists the actionable development tasks for implementing RustChain's core data structures.

## I. Common Types (`types.rs`)

- [ ] **Define `types.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] Define `Address` type (e.g., from public key hash or full public key).
- [ ] Define `PublicKey` and `Signature` types (e.g., using `ed25519-dalek` or `secp256k1` types).
- [ ] Define `Hash` type (e.g., `[u8; 32]` for SHA-256).
- [ ] Define `BlockHeight` (e.g., `u64`).
- [ ] Define `Timestamp` (e.g., `u64` for Unix timestamp).
- [ ] Define `Nonce` (e.g., `u64` for account nonce).
- [ ] Add any other primitive or widely shared types.

## II. Transaction Structure (`transaction.rs`)

- [ ] **Define `transaction.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Define `Transaction` struct.** (DEVELOPMENT_FLOW.md Phase 2)
    - Fields: `sender` (PublicKey), `recipient` (Address), `amount` (u64), `nonce` (Nonce), `signature` (Signature), `timestamp` (Timestamp).
    - Consider: payload field for future extensions (e.g. smart contracts, other tx types).
- [ ] Implement `Serialize` and `Deserialize` for `Transaction` (using `bincode` and `serde`).
- [ ] **Implement `hash_transaction()` function.** (DEVELOPMENT_FLOW.md Phase 2)
    - Should hash the serializable content of the transaction (excluding signature).
- [ ] **Define `TxValidationError` enum.** (DEVELOPMENT_FLOW.md Phase 2)
    - E.g., `InvalidSignature`, `InsufficientBalance`, `InvalidNonce`, `UnknownSender`, etc. (some might be validated by state machine).
- [ ] Implement `verify_signature()` method for `Transaction` (belongs here as it uses transaction data and signature).
- [ ] Write unit tests for `Transaction`:
    - [ ] Serialization and deserialization. (DEVELOPMENT_FLOW.md Phase 2)
    - [ ] Transaction hashing. (DEVELOPMENT_FLOW.md Phase 2)
    - [ ] Signature verification (with known keypairs). (DEVELOPMENT_FLOW.md Phase 2)
    - [ ] ✅ **Milestone Check:** Basic transaction struct integrity can be validated (hash, signature).

## III. Block Structure (`block.rs`)

- [ ] **Define `block.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Define `BlockHeader` struct.** (DEVELOPMENT_FLOW.md Phase 4)
    - Fields: `parent_hash` (Hash), `block_number` (BlockHeight), `timestamp` (Timestamp), `transactions_root` (Hash - Merkle root), `proposer` (PublicKey or Address), `signature` (Signature - block signature).
    - Consider: `state_root` (Hash) for later, `consensus_digest` (e.g. PoW difficulty, PoS VRF output).
- [ ] Implement `Serialize` and `Deserialize` for `BlockHeader`.
- [ ] Implement `hash_block_header()` function (hashes serializable content, excluding signature).
- [ ] **Define `Block` struct.** (DEVELOPMENT_FLOW.md Phase 4)
    - Fields: `header` (BlockHeader), `transactions` (Vec<Transaction>).
- [ ] Implement `Serialize` and `Deserialize` for `Block`.
- [ ] **Implement Merkle tree calculation for transactions.** (DEVELOPMENT_FLOW.md Phase 4)
    - Function: `calculate_transactions_root(transactions: &[Transaction]) -> Hash`.
    - Choose a Merkle tree library (e.g., `merkle_cbt`) or implement a simple one.
- [ ] Implement `sign_block(header_hash: Hash, private_key: &PrivateKey) -> Signature` (conceptually, signing done by validator).
- [ ] Implement `verify_block_signature(header_hash: Hash, signature: &Signature, proposer_pk: &PublicKey)`.
- [ ] Write unit tests for `Block` and `BlockHeader`:
    - [ ] Serialization and deserialization.
    - [ ] Header hashing.
    - [ ] Merkle root calculation.
    - [ ] Block signing and verification (with known keypairs).
    - [ ] ✅ **Milestone Check:** Block with header and transactions can be created, serialized, and its Merkle root and signature (if applied here) can be verified. (Corresponds to DEVELOPMENT_FLOW.md Phase 4 Milestone: "Signed block with valid Merkle root generated")

## IV. General Tasks

- [ ] Ensure all public types and functions have basic documentation comments.
- [ ] Review error handling and use appropriate error types (e.g., `TransactionValidationError`, custom errors for (de)serialization if not covered by library errors). 