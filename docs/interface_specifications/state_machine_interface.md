## State Machine Interface

**Purpose:** Defines and enforces the rules for state transitions based on transactions. It validates transactions against the current world state (e.g., balance, nonce) and applies them to update it. This module is the ultimate authority on transaction validity in the context of the current state.

**Public Functions/Methods (as a service or trait implementation):**

*   `validate_transaction_stateful(transaction: &Transaction, current_state_reader: &dyn StateReader) -> Result<(), StateError>`
    *   **Description:** Checks if a transaction is valid according to current state rules (e.g., sender has sufficient balance, transaction nonce matches account nonce, sender exists).
    *   **Parameters:**
        *   `transaction`: The transaction to validate.
        *   `current_state_reader`: A read-only accessor to the current world state.
    *   **Returns:** `Ok(())` if valid, or `StateError` detailing the validation failure.
    *   **Note:** This assumes prior syntactic validation and signature verification of the transaction.

*   `apply_block_to_state(block: &Block, state_modifier: &mut dyn StateWriter) -> Result<StateRoot, StateError>`
    *   **Description:** Atomically applies all transactions in a given block to the world state. This involves updating account balances, nonces, and potentially creating new accounts.
    *   **Parameters:**
        *   `block`: The block whose transactions are to be applied.
        *   `state_modifier`: A writable accessor to the world state, allowing modifications.
    *   **Returns:** The new `StateRoot` hash after successfully applying all transactions in the block, or `StateError` if any transaction fails or the operation is interrupted.
    *   **Preconditions:** Block has passed consensus validation (e.g., valid proposer, correct chain linkage). Transactions within the block should have been validated statefully by `validate_transaction_stateful` (or this function re-validates them before applying).
    *   **Postconditions:** If `Ok`, the world state is updated to reflect all transactions in the block. If `Err`, the state should ideally remain unchanged (atomic application).

*   `preview_transaction_effects(transaction: &Transaction, current_state_reader: &dyn StateReader) -> Result<TransactionEffects, StateError>`
    *   **Description:** (Optional) Allows for a "dry run" of a transaction to see its potential effects on state (e.g., balance changes) without actually committing them.
    *   **Parameters:** `transaction`, `current_state_reader`.
    *   **Returns:** `TransactionEffects` detailing potential state changes, or `StateError` if it would fail validation.

**Data Structures & Traits:**

*   `Transaction`, `Block` (see `../architecture/data_structures.md`)
*   `Account` (see `../architecture/state_machine.md` for its definition, or `../architecture/data_structures.md` if moved there)
*   `StateRoot` (A `Hash` representing the root of the world state, see `../architecture/data_structures.md`)
*   `TransactionEffects` (e.g., `struct { gas_used: u64, logs: Vec<LogEntry>, new_balances: HashMap<Address, u64> }` - for more advanced systems, can be simpler for basic transfers)
*   **`trait StateReader`**
    *   `get_account(address: &Address) -> Result<Option<Account>, StateError>`
    *   `get_nonce(address: &Address) -> Result<Option<u64>, StateError>`
    *   `get_balance(address: &Address) -> Result<Option<u64>, StateError>`
    *   *(Other read-only state access methods as needed)*
*   **`trait StateWriter: StateReader`** (Inherits from StateReader)
    *   `set_account(address: &Address, account: &Account) -> Result<(), StateError>`
    *   `increment_nonce(address: &Address) -> Result<(), StateError>`
    *   `update_balance(address: &Address, new_balance: u64) -> Result<(), StateError>`
    *   *(Other state modification methods as needed)*

**Error Handling:**

*   `enum StateError {`
    *   `AccountNotFound(Address),`
    *   `InsufficientBalance { required: u64, available: u64 },`
    *   `InvalidNonce { expected: u64, actual: u64 },`
    *   `SignatureVerificationFailed,` // If re-verified here
    *   `ApplyTransactionFailed(String),` // Generic for other tx application issues
    *   `StorageError(String),` // If state operations trigger underlying storage errors
    *   `InternalError(String)`
    *   `}` 