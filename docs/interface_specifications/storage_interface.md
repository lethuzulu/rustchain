## Storage Layer Interface

**Purpose:** Manages the persistent storage of blockchain data, including blocks (headers and bodies), account state, and chain metadata (like the current tip). It provides an abstraction over the underlying database (e.g., RocksDB).

**Public Functions/Methods (as a service or trait implementation):**

*   **`init_storage(config: StorageConfig) -> Result<Self, StorageError>`**
    *   **Description:** Initializes the storage system at the given path or using the provided configuration. Creates the database if it doesn't exist.
    *   **Parameters:** `config`: Configuration for storage, including the database path.
    *   **Returns:** An instance of the storage service/handler, or `StorageError` if initialization fails.

*   **`get_block_by_hash(hash: &Hash) -> Result<Option<Block>, StorageError>`**
    *   **Description:** Retrieves a full block (header and transactions) from storage by its hash.
    *   **Parameters:** `hash`: The hash of the block to retrieve.
    *   **Returns:** `Ok(Some(Block))` if found, `Ok(None)` if not found, or `StorageError` on DB/deserialization issues.

*   **`get_block_by_height(height: u64) -> Result<Option<Block>, StorageError>`**
    *   **Description:** Retrieves a full block from storage by its height. Requires an index from height to hash if not natively supported by DB.
    *   **Parameters:** `height`: The block number.
    *   **Returns:** `Ok(Some(Block))`, `Ok(None)`, or `StorageError`.

*   **`get_header_by_hash(hash: &Hash) -> Result<Option<BlockHeader>, StorageError>`**
    *   **Description:** Retrieves only the block header from storage by its hash.
    *   **Parameters:** `hash`: The hash of the block whose header is to be retrieved.
    *   **Returns:** `Ok(Some(BlockHeader))`, `Ok(None)`, or `StorageError`.

*   **`save_block(block: &Block) -> Result<(), StorageError>`**
    *   **Description:** Persists a full block (header and transactions) to storage. This should typically be an atomic operation regarding this block's data. It might also update a height-to-hash index.
    *   **Parameters:** `block`: The block to save.
    *   **Returns:** `Ok(())` on success, or `StorageError` if persistence fails.
    *   **Preconditions:** Block is fully validated and accepted by consensus and state machine.
    *   **Postconditions:** Block data is durably stored.

*   **`get_account_state(address: &Address) -> Result<Option<Account>, StorageError>`**
    *   **Description:** Retrieves the state (e.g., balance, nonce) of an account from the persisted world state.
    *   **Parameters:** `address`: The account address.
    *   **Returns:** `Ok(Some(Account))` if found, `Ok(None)` if account doesn't exist, or `StorageError`.

*   **`batch_update_account_states(updates: HashMap<Address, Account>) -> Result<(), StorageError>`**
    *   **Description:** Atomically updates the states of multiple accounts. This is useful when applying a block's transactions.
    *   **Parameters:** `updates`: A map of addresses to their new `Account` states.
    *   **Returns:** `Ok(())` or `StorageError`.
    *   **Postconditions:** All account states in `updates` are persisted, or none are if an error occurs (atomicity).

*   **`get_chain_tip() -> Result<Option<(Hash, u64)>, StorageError>`**
    *   **Description:** Retrieves the hash and height of the current canonical chain tip from metadata storage.
    *   **Returns:** `Ok(Some((tip_hash, tip_height)))`, `Ok(None)` if chain is empty/uninitialized, or `StorageError`.

*   **`set_chain_tip(hash: &Hash, height: u64) -> Result<(), StorageError>`**
    *   **Description:** Sets/updates the current canonical chain tip in metadata storage.
    *   **Parameters:** `hash`, `height`.
    *   **Returns:** `Ok(())` or `StorageError`.

**Data Structures:**

*   `Block`, `BlockHeader`, `Transaction`, `Address`, `Hash` (see `../architecture/data_structures.md`)
*   `Account` (see `../architecture/state_machine.md` or `../architecture/data_structures.md`)
*   `StorageConfig` (e.g., `struct { db_path: PathBuf, create_if_missing: bool }`)

**Error Handling:**

*   `enum StorageError {`
    *   `InitializationFailed(String),`
    *   `PathAccessError(String),`
    *   `DatabaseReadError(String),`
    *   `DatabaseWriteError(String),`
    *   `DeserializationError(String),`
    *   `SerializationError(String),`
    *   `DataNotFound(String),` // E.g., block hash or account not found
    *   `InconsistentState(String),` // E.g., height index broken
    *   `UnsupportedOperation(String)`
    *   `}` 