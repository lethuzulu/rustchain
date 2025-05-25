# 🧱 RustChain: A Custom Proof-of-Stake Blockchain Built from Scratch in Rust

> **Development Roadmap & Task Breakdown:**
> For a detailed, actionable breakdown of development phases and tasks, see [docs/development_breakdown/ROADMAP.md](docs/development_breakdown/ROADMAP.md).

RustChain is a minimalist Layer 1 blockchain protocol built entirely in **Rust**, designed to showcase deep blockchain engineering knowledge — from cryptography and P2P networking to consensus and state transitions. This is not a dApp or smart contract playground — this is the **chain itself**.

> ⚙️ Featuring: Custom PoS consensus, libp2p networking, Merkle-tree verified blocks, digital signatures, and persistent chain state.

---

## 🔍 Project Goals

- Build a functioning blockchain node that validates, produces, and syncs blocks.
- Explore core primitives like transactions, cryptographic keys, Merkle trees, and consensus.
- Learn and demonstrate low-level concepts like state management, fork resolution, and validator election.
- Avoid overreliance on existing frameworks — do it the hard way.

---

## 🚀 Features

✅ Cryptographic keypair generation (Ed25519 / Secp256k1)  
✅ Transaction signing & verification  
✅ Merkle Tree construction for block validation  
✅ Peer-to-peer networking using `libp2p`  
✅ Mempool for unconfirmed transactions  
✅ Lightweight Proof-of-Stake consensus engine  
✅ RocksDB-based persistent chain and state storage  
✅ Multiple nodes syncing & producing blocks in a local testnet

---

## 📂 Folder Structure

```plaintext
src/
├── main.rs                # Entry point – runs the full node
├── config.rs              # Config & CLI options
├── wallet/                # Wallet CLI + keypair mgmt
├── network/               # Peer-to-peer message handling
├── mempool/               # Transaction pool management
├── consensus/             # Validator selection + PoS logic
├── blockchain/            # Chain logic, block validation, state transitions
├── storage/               # RocksDB-based chain state storage
└── types.rs               # Shared data types (Address, Tx, Block, etc.)

```

## 🧪 Local Testnet Instructions

### 🔧 Build the node:

```bash
cargo build --release
```

### 🧪 Run a node:

```bash
cargo run -- --node-id 1 --port 3001
```

Run multiple instances with different ports and node IDs to simulate a local testnet.

### 🔐 Generate wallet keys:

```bash
cargo run -- wallet generate
```

### 💸 Submit a signed transaction:

```bash
cargo run -- wallet send --from <ADDR> --to <ADDR> --amount 50
```

## 🔁 Workflow

1. Nodes communicate using libp2p
2. Transactions are broadcast to peers → validated → placed in mempool
3. A PoS validator is selected to propose the next block
4. Block is built from the mempool, signed, and broadcast
5. All peers verify, apply state transitions, and append to chain

## 📚 Learning Focus

This project was built while studying:

- Blockchain data structures (blocks, Merkle trees, UTXO/account models)
- Asynchronous networking in Rust using `tokio` and `libp2p`
- Cryptography: signature schemes and hashing
- Consensus algorithms (PoS, BFT principles)
- RocksDB integration for high-performance state persistence
- Rust patterns for safe, modular system design

## 📹 Demo Video

🚧 *Coming Soon* – Will show live transaction propagation, block validation, and state syncing across 3 local nodes.

---

## 🧠 Stretch Goals (Planned)

- ✅ Transaction fees + incentives
- ⏳ Light client protocol (Merkle proof validation)
- ⏳ WASM VM for smart contract execution
- ⏳ JSON-RPC or gRPC API
- ⏳ Block explorer (Next.js frontend)

## 👨‍💻 Author

**[Your Name]** – Rust developer passionate about low-level systems, blockchain architecture, and cryptographic protocols.

[GitHub Profile](https://github.com/your-github)

[LinkedIn](https://linkedin.com/in/your-profile)

## 📄 License

MIT License. Free to use, fork, or contribute.

🧠 *This is not a tutorial copy-paste chain — it's a ground-up design to understand and demonstrate how blockchains actually work*