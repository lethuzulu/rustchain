# 💳 Wallet Interface

RustChain includes a CLI-based wallet for transaction signing and submission.

## 🔐 Keys

- Cryptography: Ed25519 for key pairs and signatures.
- Storage: Private keys stored locally as raw bytes (for simplicity in the minimal version; not for production use).
- Address Derivation: Public key → SHA-256 hash → first 20 bytes = Address.

## 📋 Commands

- `wallet generate` → create keypair
- `wallet address` → show address
- `wallet balance` → query node
- `wallet send --to <addr> --amount <amt>` → sign & send

## 🧾 Transaction Flow

1. Wallet queries nonce and balance from a configured node via JSON-RPC.
2. Signs tx locally.
3. Submits signed transaction to the node via JSON-RPC (e.g., `submit_transaction` endpoint).

## ⚙️ Configuration

- Requires the RPC endpoint address of a RustChain node for communication.
