# 📦 Core Data Structures

**Note:** All structures intended for network transmission or persistent storage will be serialized using `bincode` for efficient Rust-to-Rust representation.

## �� `Transaction`

```rust
struct Transaction {
    sender: Address,
    recipient: Address,
    amount: u64,
    nonce: u64,
    signature: Signature,
}

```

- Nonce prevents replay
- Signature must cover all fields (except `signature`)
- Hash = SHA-256 over serialized data

## 🧱 `BlockHeader`

```rust
struct BlockHeader {
    parent_hash: Hash,          // Hash of the previous block's header
    block_number: u64,
    timestamp: u64,             // Unix timestamp (seconds since epoch)
    tx_root: Hash,              // Merkle root of transactions in the block body
    validator: Address,         // Public address of the block's proposer/validator
    signature: Signature,       // Validator's signature over the canonical hash of the header (excluding this signature field itself)
}

```

## 📦 `Block`

```rust
struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

```

## 🌲 Merkle Tree

- `tx_root` = Merkle root over all txs
- Used for tx integrity
- Future: Merkle proofs for SPV

## ⚙️ Type Aliases & Cryptographic Primitives

- **`Address`**: `[u8; 32]` (e.g., SHA-256 hash of a public key, or the public key bytes directly if using Ed25519 which has 32-byte public keys).
- **`Hash`**: `[u8; 32]` (e.g., SHA-256 output).
- **`Signature`**: `Vec<u8>` (Specific format depends on the chosen cryptographic library, e.g., Ed25519 signature).
- **Hashing Algorithm:** SHA-256 is used for block hashes, Merkle roots, and transaction hashes.
- **Digital Signature Algorithm:** Ed25519 (as chosen for wallets and validator signing).
