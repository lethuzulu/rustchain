## Wallet RPC Interface (Node-Side)

**Purpose:** Defines the JSON-RPC endpoints that a RustChain node exposes for CLI wallets (and potentially other clients) to interact with the blockchain.

*(To be detailed: Defines the specific JSON-RPC methods, their request parameters, and response formats. This is less about Rust traits/functions and more about the network API contract.)*

**Endpoints / Methods:**

*   **`get_balance`**
    *   **Description:** Retrieves the balance of a given account address.
    *   **Request Parameters:**
        *   `address: String` (Hex-encoded account address)
    *   **Response:**
        *   `balance: u64`
        *   `error: Option<RpcError>`
    *   **Example:** `{"jsonrpc": "2.0", "method": "get_balance", "params": {"address": "0x..."}, "id": 1}`

*   **`get_nonce`**
    *   **Description:** Retrieves the current nonce for a given account address.
    *   **Request Parameters:**
        *   `address: String` (Hex-encoded account address)
    *   **Response:**
        *   `nonce: u64`
        *   `error: Option<RpcError>`

*   **`submit_transaction`**
    *   **Description:** Submits a new, signed transaction to the node's mempool.
    *   **Request Parameters:**
        *   `transaction_hex: String` (Hex-encoded serialized signed transaction)
    *   **Response:**
        *   `transaction_hash: String` (Hex-encoded hash of the submitted transaction if accepted)
        *   `error: Option<RpcError>` (e.g., if transaction is invalid, malformed, or mempool is full)

*   **`get_transaction_status`** (Optional)
    *   **Description:** Retrieves the status of a previously submitted transaction.
    *   **Request Parameters:**
        *   `transaction_hash: String` (Hex-encoded transaction hash)
    *   **Response:**
        *   `status: String` (e.g., "Pending", "InBlock", "Confirmed", "Failed")
        *   `block_hash: Option<String>` (If included in a block)
        *   `error: Option<RpcError>`

*   **`get_latest_block_info`** (Optional)
    *   **Description:** Retrieves information about the latest block(s).
    *   **Request Parameters:** (e.g., `count: Option<u32>`)
    *   **Response:** (e.g., array of block summaries: `hash`, `height`, `timestamp`)
        *   `error: Option<RpcError>`

**Data Structures (JSON):**

*   Follows JSON-RPC 2.0 specification.
*   Specific request and response objects for each method (as outlined above).

**Error Handling (JSON-RPC):**

*   `RpcError` object with `code` and `message` fields, adhering to JSON-RPC error standards.
    *   Examples: `InvalidParams`, `InternalError`, `TransactionRejected`, `ResourceNotFound`. 