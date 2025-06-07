# 🔄 State Machine

This document describes the **world state representation** and **ledger rules** for updating that state based on transactions and blocks.

The state machine defines the business logic of RustChain — how balances, nonces, and validator-related state evolve as new transactions and blocks are processed.

---

## 🗂️ World State Layout

Each address in the system maps to an `Account` record.

```rust
struct Account {
    balance: u64,
    nonce: u64,
    // future: stake: Option<u64>,
}

```

- `balance`: total tokens held
- `nonce`: incremented with each valid outgoing transaction
- Future extensions may include staking information, contract storage, etc.

---

## 🔁 State Transition Logic

When a block is applied, the node executes each transaction in order and updates the world state accordingly.

### ✅ Transaction Validation

A transaction is valid only if:

1. `signature` is valid for `sender`
2. `nonce` equals the account's current nonce
3. `sender.balance >= amount`

Invalid transactions are **rejected**, and the entire block is **invalidated** if any tx is bad.

---

### 🪙 Transaction Execution

For each valid transaction:

1. `sender.balance -= amount`
2. `sender.nonce += 1`
3. `recipient.balance += amount`

If recipient does not exist in state, a new `Account { balance = amount, nonce = 0 }` is created.

---

### 🧱 Block Application Flow

1. Validate block signature (matches expected validator)
2. Validate Merkle root matches transactions
3. Apply each transaction using the rules above
4. Commit updated world state to the storage layer

---

## 🌐 Validator-Specific Logic (Optional/Future)

- Stake tracking (for PoS weight)
- Slashing conditions (e.g., double signing)
- Epoch rotation or validator elections

These would extend the state machine but are **not part of the minimal prototype**.

---

## 🧪 Future Features

- Smart contract storage trie (key-value per address)
- Account-based vs UTXO model abstraction
- Multi-token or native assets
