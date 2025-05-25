## Mempool Interface

**Purpose:** Manages a pool of unconfirmed transactions, providing transactions for block creation and ensuring basic transaction validity (e.g., syntax, signature, non-replayability within mempool context) before wider validation by the State Machine.

**Public Functions/Methods:**

*   **`init_mempool(config: MempoolConfig) -> Result<Self, MempoolError>`**
    *   **Description:** Initializes a new mempool instance.
    *   **Parameters:** `config`: Configuration for the mempool (e.g., max size, transaction expiry, etc.).
    *   **Returns:** A new mempool instance or `MempoolError`.

*   **`add_transaction(transaction: Transaction) -> Result<(), MempoolError>`**
    *   **Description:** Attempts to add a transaction to the mempool. This typically involves initial validation (e.g., format, signature, not already in pool).
    *   **Parameters:** `transaction`: The transaction to add.
    *   **Returns:** `Ok(())` if successfully added, or `MempoolError` if rejected.
    *   **Preconditions:** Transaction should be well-formed.
    *   **Postconditions:** If successful, transaction is in the pool and pending inclusion in a block.

*   **`get_pending_transactions(max_count: usize, max_total_size_bytes: usize) -> Result<Vec<Transaction>, MempoolError>`**
    *   **Description:** Retrieves a list of pending transactions suitable for inclusion in a new block, up to specified limits.
    *   **Parameters:**
        *   `max_count`: Maximum number of transactions to return.
        *   `max_total_size_bytes`: Maximum total serialized size of transactions to return.
    *   **Returns:** A vector of transactions, or `MempoolError`.

*   **`remove_transactions(transaction_ids: &[TransactionIdentifier]) -> Result<(), MempoolError>`**
    *   **Description:** Removes transactions from the mempool, typically after they have been included in a confirmed block.
    *   **Parameters:** `transaction_ids`: A slice of identifiers for transactions to remove.
    *   **Returns:** `Ok(())` or `MempoolError` (e.g., if some transactions were not found).

*   **`contains_transaction(transaction_id: &TransactionIdentifier) -> bool`**
    *   **Description:** Checks if a transaction with the given identifier is currently in the mempool.
    *   **Parameters:** `transaction_id`: The identifier of the transaction to check.
    *   **Returns:** `true` if the transaction is in the pool, `false` otherwise.

**Data Structures:**

*   `Transaction` (see `../architecture/data_structures.md`)
*   `TransactionIdentifier` (Typically `Hash` of the transaction, see `../architecture/data_structures.md`)
*   `MempoolConfig` (e.g., `struct { max_transactions: usize, max_size_bytes: usize, transaction_ttl_seconds: u64 }`)

**Error Handling:**

*   `enum MempoolError {`
    *   `InitializationFailed(String),`
    *   `TransactionInvalidFormat,`
    *   `TransactionSignatureInvalid,`
    *   `TransactionNonceTooLow,`       // If mempool tracks local nonces for accounts
    *   `TransactionAlreadyExists,`
    *   `PoolFullMaxTransactions,`
    *   `PoolFullMaxSize,`
    *   `TransactionExpired,`
    *   `InternalError(String)`
    *   `}`