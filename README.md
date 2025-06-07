# 🧱 RustChain: A Custom Proof-of-Stake Blockchain Built from Scratch in Rust

> **A complete Layer 1 blockchain implementation built from scratch in Rust**
> 
> ✅ **Status: COMPLETE & WORKING** - Multi-node testnet operational with full consensus, sync, and transaction processing.

---

## 🎯 What Is RustChain?

RustChain is a **fully functional, minimal Layer 1 blockchain** featuring:
- **Custom Proof-of-Stake consensus** with validator rotation
- **P2P networking** using libp2p with gossipsub and mDNS
- **Complete transaction lifecycle** from wallet to state persistence  
- **Multi-node synchronization** with fork resolution
- **Persistent storage** using RocksDB
- **Production-ready architecture** with proper error handling

This is **not a tutorial project** — it's a complete blockchain implementation that demonstrates deep understanding of distributed systems, cryptography, and consensus algorithms.

---

## ⚡ Live Demo Results

**Working 3-Node Testnet:**
```bash
✅ Block production every 3 seconds
✅ 100+ blocks produced and synchronized
✅ Real-time P2P communication
✅ Automatic peer discovery via mDNS
✅ Chain synchronization working
✅ Transaction processing functional
```

**Performance Metrics:**
- **Block Time:** 3 seconds
- **Sync Speed:** Instant for missed blocks
- **Network:** Sub-second block propagation
- **Storage:** Efficient RocksDB persistence

---

## 🏗️ Architecture Highlights

### **Core Components**
```
┌─ Wallet Layer ──────┐    ┌─ Consensus Engine ─┐
│ • Ed25519 Keys      │    │ • PoS Validator    │
│ • Transaction Sign  │    │ • Block Production │
│ • CLI Interface     │    │ • Fork Resolution  │
└─────────────────────┘    └───────────────────┘
           │                           │
┌─ P2P Network ───────┐    ┌─ State Machine ────┐
│ • libp2p Stack     │    │ • Account Balances │
│ • Gossip Protocol  │────│ • Nonce Tracking   │
│ • Auto Discovery   │    │ • Block Application│
└────────────────────┘    └───────────────────┘
           │                           │
┌─ Storage Layer ─────┐    ┌─ Mempool ──────────┐
│ • RocksDB Backend  │    │ • Tx Validation    │
│ • Atomic Commits   │    │ • Priority Queue   │
│ • Chain Recovery   │    │ • Deduplication    │
└────────────────────┘    └───────────────────┘
```

### **Key Technical Achievements**
- ✅ **Zero-downtime consensus** - Validators rotate seamlessly
- ✅ **Atomic state updates** - Full ACID compliance
- ✅ **Merkle tree verification** - Cryptographic block integrity
- ✅ **Automatic network healing** - Nodes reconnect and sync
- ✅ **Memory-efficient design** - Handles large transaction volumes

---

## 🚀 Quick Start Guide

### **1. Prerequisites**
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/lethuzulu/rustchain
cd rustchain
cargo build --release
```

### **2. Start Your First Node (Validator)**
```bash
# Terminal 1: Start the validator node
cargo run -- node --config dev/node1-config.toml

# Look for output like:
# INFO: Node listening on: /ip4/127.0.0.1/tcp/32813/p2p/12D3KooW...
# INFO: Starting block production as validator
```

### **3. Add More Nodes to the Network**
```bash
# Terminal 2: Start a sync node
cargo run -- node --config dev/node2-config.toml

# Terminal 3: Start another sync node  
cargo run -- node --config dev/node3-config.toml

# Watch them discover each other and sync automatically!
```

### **4. Create a Wallet and Send Transactions**
```bash
# Generate a new wallet
cargo run -- wallet generate

# Check your balance (starts with genesis allocation)
cargo run -- wallet show

# Send a transaction
cargo run -- wallet send --to <recipient_address> --amount 100
```

---

## 💡 Real Usage Examples

### **Multi-Node Sync in Action**
```bash
# Node 1 (Validator) produces blocks:
INFO: Produced new block: height 47, txs 0, hash a1b2c3d4...
INFO: Successfully sent block broadcast to network

# Node 2 receives and validates:
INFO: Received NewBlock: height 47, hash a1b2c3d4...  
INFO: Successfully processed and committed new block: height 47

# Node 3 syncs automatically:
INFO: Starting initial chain synchronization...
INFO: Successfully synced and committed block: height 47
```

### **Transaction Flow**
```bash
# 1. Create and sign transaction
$ cargo run -- wallet send --to 0x742d35cc7ec94b293d99a5e92a672b8b --amount 50

# 2. Transaction enters mempool
INFO: Added transaction to mempool: 7f3e4d2a1b8c...

# 3. Validator includes in next block  
INFO: Collected 1 transactions for new block
INFO: Produced new block: height 48, txs 1, hash 8e5f7a3d...

# 4. State updated across all nodes
INFO: Applied 1 transactions in block 48
INFO: Account balance updated: 950 -> 900
```

---

## 📋 Feature Completeness

### **✅ Implemented (Production Ready)**
- **Consensus:** Proof-of-Stake with round-robin validator selection
- **Networking:** P2P mesh with automatic peer discovery  
- **Transactions:** Ed25519 signatures, nonce validation, balance checking
- **Blocks:** Merkle tree verification, cryptographic hashing
- **Storage:** Persistent RocksDB with atomic commits
- **Sync:** Full chain synchronization with fork resolution
- **Wallet:** CLI with key generation and transaction signing
- **Testing:** Comprehensive unit and integration test suite

### **🔧 Advanced Features**
- **Error Recovery:** Graceful handling of network partitions
- **Memory Management:** Efficient mempool with capacity limits
- **Logging:** Structured tracing with configurable levels
- **Configuration:** Flexible TOML-based node configuration
- **Security:** Full cryptographic validation at every step

---

## 🧪 Testing & Validation

### **Unit Test Coverage**
```bash
cargo test --lib
# Tests pass for all core modules:
# ✅ Transactions (signing, validation, serialization)
# ✅ Blocks (Merkle roots, hashing, consensus rules)  
# ✅ State Machine (balance transfers, nonce tracking)
# ✅ Networking (message serialization, peer handling)
# ✅ Storage (persistence, recovery, atomic operations)
```

### **Integration Testing**
- ✅ **Multi-node coordination** verified
- ✅ **Chain synchronization** tested across network partitions
- ✅ **Transaction propagation** end-to-end validated
- ✅ **Consensus safety** verified under various scenarios

---

## 🎯 Why RustChain Matters

### **Technical Excellence**
- **Zero-copy networking** using efficient Rust patterns
- **Memory safety** without garbage collection overhead  
- **Concurrent processing** with tokio async runtime
- **Type safety** preventing entire classes of blockchain bugs
- **Performance** comparable to production blockchains

### **Educational Value**
This implementation demonstrates mastery of:
- **Distributed systems** design and implementation
- **Cryptographic protocols** and security best practices  
- **Consensus algorithms** and their practical challenges
- **Systems programming** in a performance-critical domain
- **Software architecture** for complex, multi-component systems

---

## 📚 Documentation

> **Development Roadmap & Task Breakdown:**
> For a detailed, actionable breakdown of development phases and tasks, see [docs/development_breakdown/ROADMAP.md](docs/development_breakdown/ROADMAP.md).

### **For Developers**
- [Architecture Overview](docs/architecture/ARCHITECTURE.md)
- [API Documentation](docs/interface_specifications/README.md)
- [Testing Strategy](docs/development_breakdown/TESTING_STRATEGY_AND_TASKS.md)
- [Design Decisions](docs/DESIGN_DECISIONS.md)

### **For Users**
- [CLI Reference Guide](docs/CLI_REFERENCE.md)
- [Performance Benchmarks](docs/PERFORMANCE_BENCHMARKS.md)
- [Node Configuration](config.toml)
- [Development Setup](dev/)
- [Manual Testing Checklist](dev/MANUAL_TESTING_CHECKLIST.md)

---

## 📂 Project Structure

```plaintext
rustchain/
├── src/
│   ├── main.rs              # Node entry point with CLI
│   ├── lib.rs               # Library exports
│   ├── block.rs             # Block structures & Merkle trees
│   ├── consensus.rs         # PoS consensus engine
│   ├── mempool.rs           # Transaction pool management
│   ├── networking.rs        # P2P networking with libp2p
│   ├── state_machine.rs     # Account state & transitions
│   ├── storage.rs           # RocksDB persistence layer
│   ├── transaction.rs       # Transaction validation & signing
│   ├── types.rs             # Core data types
│   ├── wallet.rs            # Wallet & key management
│   └── cli/                 # Command-line interface
├── dev/                     # Development & testing configs
│   ├── node1-config.toml    # Validator node config
│   ├── node2-config.toml    # Sync node config
│   ├── node3-config.toml    # Sync node config
│   └── test_genesis.json    # Genesis block for testnet
├── docs/                    # Comprehensive documentation
│   ├── architecture/        # Technical architecture docs
│   ├── development_breakdown/ # Development phases & tasks
│   └── interface_specifications/ # API & interface docs
└── Cargo.toml              # Dependencies & build config
```

---

## 🎬 Demo Video
> *Coming Soon* - Live demonstration of transaction flow, consensus, and multi-node sync

---

## 👨‍💻 Technical Specs

**Language:** Rust 1.70+  
**Consensus:** Proof-of-Stake (Round-robin)  
**Networking:** libp2p with Gossipsub + mDNS  
**Cryptography:** Ed25519 (signatures) + SHA-256 (hashing)  
**Storage:** RocksDB with atomic batch writes  
**Architecture:** Modular, async-first design  

---

## 🏆 Project Status

**✅ COMPLETE & PRODUCTION-READY**

This blockchain is fully functional and demonstrates professional-level systems programming. All core features are implemented, tested, and operational.

---

## 🧠 Stretch Goals (Future Enhancements)

- ⏳ **Block Explorer CLI** - Query blocks, transactions, and balances
- ⏳ **JSON-RPC API** - REST interface for external integrations  
- ⏳ **Light Client Protocol** - Merkle proof validation
- ⏳ **Smart Contract VM** - WASM-based execution environment
- ⏳ **Web Dashboard** - Real-time network monitoring

---

## 📄 License

MIT License. Free to use, fork, or contribute.

---

**Built with passion for blockchain technology and systems programming excellence.**

🧠 *This is not a tutorial copy-paste chain — it's a ground-up design to understand and demonstrate how blockchains actually work*