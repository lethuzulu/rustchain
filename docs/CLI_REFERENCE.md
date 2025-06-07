# 📖 RustChain CLI Reference

Complete command-line interface documentation for RustChain blockchain operations.

---

## 🚀 Quick Navigation

- [Node Operations](#-node-operations)
- [Wallet Management](#-wallet-management)
- [Development Tools](#-development-tools)
- [Configuration](#-configuration)
- [Examples](#-examples)

---

## 🔧 Node Operations

### **Start a Node**

```bash
cargo run -- node [OPTIONS]
```

**Options:**
- `--config <FILE>` - Specify configuration file (default: `config.toml`)
- `--data-dir <DIR>` - Set database directory (default: `rustchain_db/`)
- `--log-level <LEVEL>` - Set logging level (`trace`, `debug`, `info`, `warn`, `error`)

**Examples:**
```bash
# Start with default config
cargo run -- node

# Start validator node with custom config
cargo run -- node --config dev/node1-config.toml

# Start with debug logging
cargo run -- node --log-level debug --config dev/node2-config.toml
```

**Expected Output:**
```
INFO rustchain: Starting RustChain node...
INFO rustchain: Loaded genesis block with 1 accounts
INFO rustchain::networking: Node listening on: /ip4/127.0.0.1/tcp/32813/p2p/12D3KooW...
INFO rustchain: Starting block production as validator
INFO rustchain: Node startup completed successfully
```

---

## 💰 Wallet Management

### **Generate New Wallet**

```bash
cargo run -- wallet generate [OPTIONS]
```

**Options:**
- `--output <FILE>` - Save wallet to specific file (default: `wallet.key`)
- `--format <FORMAT>` - Output format (`json`, `hex`) (default: human-readable)

**Examples:**
```bash
# Generate new wallet (saves to wallet.key)
cargo run -- wallet generate

# Generate wallet with custom filename
cargo run -- wallet generate --output my-wallet.key

# Generate wallet with JSON output
cargo run -- wallet generate --format json
```

**Expected Output:**
```
✅ Generated new wallet successfully!

🔑 Wallet Details:
   Address: 0x68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2
   Public Key: 68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2
   Private Key: [HIDDEN - saved to wallet.key]

⚠️  Important: Backup your wallet.key file securely!
```

### **Show Wallet Information**

```bash
cargo run -- wallet show [OPTIONS]
```

**Options:**
- `--wallet <FILE>` - Specify wallet file (default: `wallet.key`)
- `--balance` - Query current balance from network
- `--node <URL>` - Connect to specific node for balance query

**Examples:**
```bash
# Show wallet info (no balance query)
cargo run -- wallet show

# Show wallet with current balance
cargo run -- wallet show --balance

# Show specific wallet file
cargo run -- wallet show --wallet dev/validator.key --balance
```

**Expected Output:**
```
💼 Wallet Information:
   Address: 0x68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2
   Public Key: 68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2
   Current Balance: 1000 RUST
   Nonce: 0
```

### **Send Transaction**

```bash
cargo run -- wallet send [OPTIONS] --to <ADDRESS> --amount <AMOUNT>
```

**Required Options:**
- `--to <ADDRESS>` - Recipient address (hex format)
- `--amount <AMOUNT>` - Amount to send (integer)

**Optional:**
- `--wallet <FILE>` - Sender wallet file (default: `wallet.key`)
- `--nonce <NONCE>` - Override automatic nonce (advanced)
- `--dry-run` - Validate transaction without broadcasting

**Examples:**
```bash
# Send 100 tokens
cargo run -- wallet send \
  --to 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000 \
  --amount 100

# Send with custom wallet
cargo run -- wallet send \
  --wallet my-wallet.key \
  --to 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000 \
  --amount 50

# Dry run (validate only)
cargo run -- wallet send \
  --to 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000 \
  --amount 25 \
  --dry-run
```

**Expected Output:**
```
📤 Sending Transaction...
   From: 0x68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2
   To: 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000
   Amount: 100 RUST
   Nonce: 0

✅ Transaction sent successfully!
   Transaction ID: 0x7f3e4d2a1b8c9e5f2a6d8c4e9f1a3b7c5d2e8f4a9b6c3e7f1d5a8c2e6f9b4d7a
   Status: Pending (waiting for inclusion in block)
```

---

## 🛠️ Development Tools

### **Create Validator Key**

```bash
cargo run --bin create_validator_key [OUTPUT_FILE]
```

**Examples:**
```bash
# Create validator key (saves to validator.key)
cargo run --bin create_validator_key

# Create with custom filename
cargo run --bin create_validator_key dev/my-validator.key
```

### **Debug Genesis File**

```bash
cargo run --bin debug_genesis [GENESIS_FILE]
```

**Examples:**
```bash
# Debug default genesis
cargo run --bin debug_genesis

# Debug custom genesis file
cargo run --bin debug_genesis dev/test_genesis.json
```

### **Debug Configuration**

```bash
cargo run --bin debug_toml [CONFIG_FILE]
```

**Examples:**
```bash
# Debug default config
cargo run --bin debug_toml

# Debug specific config
cargo run --bin debug_toml dev/node1-config.toml
```

---

## ⚙️ Configuration

### **Configuration File Format (TOML)**

```toml
# Node Configuration
[node]
is_validator = true
validator_key_path = "dev/validator1.key"
data_dir = "test_node_1_db"

# Network Configuration  
[network]
listen_address = "/ip4/0.0.0.0/tcp/0"
bootstrap_peers = []

# Genesis Configuration
[genesis]
file_path = "dev/test_genesis.json"

# Logging Configuration
[logging]
level = "info"
```

### **Genesis File Format (JSON)**

```json
{
  "accounts": [
    {
      "address": "68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2",
      "balance": 1000,
      "nonce": 0
    }
  ],
  "validators": [
    "68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2"
  ]
}
```

---

## 💡 Examples

### **Complete Local Testnet Setup**

```bash
# Terminal 1: Start validator node
cargo run -- node --config dev/node1-config.toml

# Terminal 2: Start sync node
cargo run -- node --config dev/node2-config.toml

# Terminal 3: Start another sync node
cargo run -- node --config dev/node3-config.toml

# Terminal 4: Create wallet and send transaction
cargo run -- wallet generate
cargo run -- wallet send \
  --to 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000 \
  --amount 100
```

### **Monitor Network Activity**

```bash
# Watch block production with detailed logs
RUST_LOG=info cargo run -- node --config dev/node1-config.toml

# Track specific modules
RUST_LOG=rustchain::consensus=debug,rustchain::networking=info cargo run -- node
```

### **Transaction Batch Processing**

```bash
# Send multiple transactions
for i in {1..5}; do
  cargo run -- wallet send \
    --to 0x742d35cc7ec94b293d99a5e92a672b8b00000000000000000000000000000000 \
    --amount $((i * 10))
  sleep 1
done
```

---

## 🔍 Command Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Configuration error |
| `3` | Network error |
| `4` | Wallet error |
| `5` | Transaction error |

---

## 📝 Output Formats

### **Human-Readable (Default)**
```
✅ Transaction sent successfully!
   Transaction ID: 0x7f3e4d2a...
   Status: Pending
```

### **JSON Format**
```bash
# Use --format json for machine-readable output
cargo run -- wallet show --format json
```
```json
{
  "address": "0x68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2",
  "balance": 1000,
  "nonce": 0
}
```

---

## 🛡️ Security Notes

- **Private Keys:** Never share your `wallet.key` files
- **Addresses:** Can be shared publicly for receiving transactions
- **Backups:** Always backup wallet files before operations
- **Network:** Use trusted bootstrap peers in production

---

## 🔗 See Also

- [Architecture Overview](architecture/ARCHITECTURE.md)
- [Performance Benchmarks](PERFORMANCE_BENCHMARKS.md)
- [Configuration Examples](../dev/) 