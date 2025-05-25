## Networking Layer Interface

**Purpose:** Handles all peer-to-peer communication using libp2p, including message broadcasting and reception (transactions, blocks), peer discovery, connection management, and block synchronization requests/responses.

**Key Data Structure for Messages:**

```rust
pub enum NetworkMessage {
    Transaction(Transaction),
    Block(Block),
    SyncRequest { from_height: u64, to_hash: Option<Hash> }, // Request blocks from a certain height, optionally up to a known hash
    SyncResponseBlocks { blocks: Vec<Block> },
    SyncResponseNoBlocks, // Indicates requested blocks not found or requester is up-to-date
    // Potentially other message types like GetHeaders, Headers, etc. for more efficient sync
}
```

**Public Functions/Methods (Service Interface):**

*   **`start_listening(config: NetworkConfig, message_sender: Sender<IncomingMessage>) -> Result<NetworkServiceHandle, NetworkError>`**
    *   **Description:** Initializes and starts the libp2p Swarm, begins listening for incoming connections and messages. Incoming messages are sent to the provided `message_sender` channel for processing by other modules.
    *   **Parameters:**
        *   `config`: Network configuration (e.g., listen address, bootstrap peers, node identity key).
        *   `message_sender`: A channel sender (`tokio::sync::mpsc::Sender` or similar) to pass `IncomingMessage` (which includes `PeerId` and `NetworkMessage`) to the core logic.
    *   **Returns:** A `NetworkServiceHandle` to interact with the running network service (e.g., for broadcasting), or `NetworkError`.

*   **`NetworkServiceHandle::broadcast_message(message: NetworkMessage) -> Result<(), NetworkError>`**
    *   **Description:** Submits a `NetworkMessage` (e.g., new transaction or block) to the networking layer for gossiping to connected peers.
    *   **Parameters:** `message`: The `NetworkMessage` to broadcast.
    *   **Returns:** `Ok(())` on successful submission to broadcast queue, `NetworkError` otherwise.
    *   **Preconditions:** Networking layer must be active. Message should be valid.
    *   **Postconditions:** Message is queued for broadcast.

*   **`NetworkServiceHandle::send_direct_message(peer_id: PeerId, message: NetworkMessage) -> Result<(), NetworkError>`**
    *   **Description:** Sends a `NetworkMessage` directly to a specific peer (e.g., for a `SyncRequest`).
    *   **Parameters:** `peer_id`, `message`.
    *   **Returns:** `Ok(())` or `NetworkError`.

**Incoming Messages (via `message_sender` channel):**

*   `struct IncomingMessage { peer_id: PeerId, message: NetworkMessage }`
    *   Other modules (e.g., node orchestrator) will listen on the receiver end of this channel. The orchestrator is then responsible for dispatching the `NetworkMessage` to appropriate handlers (e.g., mempool for `Transaction`, consensus for `Block`). This replaces the `register_message_handler` concept with a channel-based approach for decoupling.

**Data Structures:**

*   `Transaction`, `Block`, `Hash` (see `../architecture/data_structures.md`)
*   `PeerId` (from `libp2p`)
*   `NetworkConfig` (e.g., `struct { listen_addr: Multiaddr, bootstrap_nodes: Vec<Multiaddr>, private_key_seed: Option<String> }`)
*   `NetworkServiceHandle` (An opaque handle, possibly with methods like `shutdown()`)

**Error Handling:**

*   `enum NetworkError {`
    *   `InitializationFailed(String),`
    *   `TransportError(String),`
    *   `BroadcastFailed(String),`
    *   `DirectSendFailed(String),`
    *   `SerializationFailed(String),`
    *   `PeerUnreachable(PeerId),`
    *   `NotListening,`
    *   `ChannelSendError(String)`
    *   `}` 