# 💳 CLI Wallet: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 1: Key Management & CLI Wallet](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-1-key-management--cli-wallet)

**Likely Crates/Tools:**
- `ed25519-dalek` (for Ed25519 key generation, signing) or `secp256k1` (if chosen)
- `rand` (randomness for key generation)
- `clap` (CLI argument parsing)
- `bincode` / `serde` (for serializing transaction data to be signed/sent)
- `hex` (for displaying keys/addresses)
- `tokio` (if RPC calls are async)
- `reqwest` or `ureq` (for sending transactions to a node - future phase)

This document lists the actionable development tasks for implementing the RustChain CLI wallet.

## I. Key Management

- [ ] **Choose cryptographic scheme:** Decide between Ed25519 (recommended for simplicity/performance) or Secp256k1.
    - Update `core_data_structures_tasks.md` regarding `PublicKey` and `Signature` types if not already aligned.
- [ ] **Implement key pair generation.** (DEVELOPMENT_FLOW.md Phase 1)
    - Crate: `ed25519-dalek` + `rand`, or `secp256k1` + `rand`.
    - Function: `generate_keypair() -> (PrivateKey, PublicKey)`.
- [ ] **Implement private key storage/loading.** (DEVELOPMENT_FLOW.md Phase 1)
    - For dev-mode: Store raw private key bytes to a file (e.g., `wallet.key`).
    - Consider simple encryption/password protection if time permits (stretch goal for minimal version).
    - Functions: `save_private_key(pk: &PrivateKey, path: &Path)`, `load_private_key(path: &Path) -> Result<PrivateKey, Error>`.
- [ ] **Implement public address derivation.** (DEVELOPMENT_FLOW.md Phase 1)
    - From public key (e.g., hashing the public key, or using it directly, decide on format).
    - Function: `derive_address(public_key: &PublicKey) -> Address`.

## II. Transaction Handling

- [ ] **Implement transaction signing.** (DEVELOPMENT_FLOW.md Phase 1)
    - Function: `sign_transaction(transaction_data_to_sign: &[u8], private_key: &PrivateKey) -> Signature`.
    - Note: The `Transaction` struct itself will be defined in `core_data_structures`. This task focuses on the wallet taking the serializable parts of a transaction and signing its hash.
- [ ] Prepare transaction data for sending (serialization of the full transaction, including signature, happens via `Transaction` struct's `Serialize` trait).

## III. CLI Commands (using `clap`)

- [ ] **Setup basic CLI structure with `clap`.**
- [ ] **Implement `wallet generate` command.** (DEVELOPMENT_FLOW.md Phase 1)
    - Generates a new keypair.
    - Saves the private key to a file (e.g., `wallet.key` or user-specified).
    - Displays the public key and derived address to the console.
- [ ] **Implement `wallet show` command.** (DEVELOPMENT_FLOW.md Phase 1)
    - Loads private key from file.
    - Displays the corresponding public key and address.
    - (Future: query and display balance/nonce from a node).
- [ ] **Implement `wallet send` command.** (DEVELOPMENT_FLOW.md Phase 1)
    - Arguments: `<recipient_address>`, `<amount>`.
    - Loads private key from file.
    - Creates a transaction (gets nonce - for now, can be hardcoded or 0, later from node).
    - Signs the transaction.
    - Serializes the transaction.
    - **For now:** Prints the serialized transaction (hex-encoded) to the console or saves to a file. (Submitting to a node comes after P2P setup).
    - [ ] ✅ **Milestone Check:** A transaction can be created, signed, and serialized using CLI commands. (Corresponds to DEVELOPMENT_FLOW.md Phase 1 Milestone: "First signed transaction created from CLI")

## IV. RPC Communication (Placeholder for later integration)

- [ ] Define structure for querying node for nonce and balance (relevant for `wallet send` and `wallet show balance`).
- [ ] Define structure for submitting serialized transaction to a node (relevant for `wallet send`).

## V. Unit Tests

- [ ] Test key generation.
- [ ] Test key saving and loading.
- [ ] Test address derivation.
- [ ] Test transaction signing and verification (using public key).
- [ ] Test CLI command argument parsing (basic tests).

## VI. General Tasks

- [ ] Basic error handling for file operations, (de)serialization, signing.
- [ ] User-friendly console output for all commands. 