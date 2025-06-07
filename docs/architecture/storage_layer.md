# 🗃️ Storage Layer

This document describes how RustChain stores blockchain data persistently using RocksDB (or Sled as an alternative).

The storage layer provides disk-backed persistence for:

- Chain data (blocks, headers)
- World state (accounts)
- Metadata (chain tip, validator config)

---

## 💾 Database Backend

- RocksDB (default)
- Sled (optional/future)
- Key-value store optimized for fast reads/writes

---

## 🗂️ RocksDB Schema Layout

```text
blocks/{hash}        => Block binary
headers/{height}     => BlockHeader binary
state/{address}      => Account { balance, nonce }
meta/tip             => Latest block hash
meta/height          => Latest block height

```

Each entry is a serialized (e.g., bincode) Rust struct.

**Genesis State:** The initial state (e.g., genesis block, initial account balances if any) is established by loading from a genesis configuration file or using hardcoded parameters when the first node starts.

---

## 🧾 On Block Commit

When a block is finalized (these operations should be performed atomically, e.g., using database transactions/batches):

1. Store full `Block` under `blocks/{hash}`
2. Store `BlockHeader` under `headers/{height}`
3. For each transaction:
    - Update sender and recipient accounts in `state/{address}`
4. Update:
    - `meta/tip` to new block hash
    - `meta/height` to new height

---

## 🧠 Mempool Persistence

The mempool is currently **in-memory only** (RAM queue). On restart:

- Mempool is cleared
- Unconfirmed transactions must be re-submitted by wallets

**Future option**: persist mempool queue in `mempool/txs/{hash}` entries.

---

## 📦 Snapshotting & Pruning (Planned/Future)

Future performance enhancements may include:

- **State snapshots**: periodic dumps of full world state (for syncing or checkpoints)
- **Pruning**: discard full block bodies after N blocks, keeping only headers/state
- **Chain archival mode vs light mode**

---

## 🔒 Data Integrity

- SHA-256 hashes are stored for:
    - BlockHeader.parent_hash
    - Merkle root of transactions
- Verifying stored block hashes against header ensures tamper resistance

---

## 🧪 Debugging & CLI Read Access

The optional block explorer module reads directly from this schema to:

- Display blocks, transactions
- Inspect account state
- Export chain segments
