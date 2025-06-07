# 🌐 P2P Networking (libp2p): Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 8: Networking Layer (libp2p)](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-8-networking-layer--libp2p)
- Integration with [Phase 10: Basic Chain Sync](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-10-basic-chain-sync)

**Likely Crates/Tools:**
- `libp2p` (core library, including transports, security, multiplexing, gossipsub, request-response, identify, mdns)
- `tokio` (async runtime)
- `bincode` & `serde` (for message serialization/deserialization)
- `tracing` (for logging P2P events, errors, and message flows)

This document lists actionable tasks for implementing RustChain's P2P networking layer.

## I. Module Definition & Basic libp2p Setup

- [ ] **Define `networking.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Implement PeerId generation/loading.**
    - Crate: `libp2p-identity`.
    - Store/load peer keypair to reuse `PeerId`.
- [ ] **Setup libp2p Swarm.** (DEVELOPMENT_FLOW.md Phase 8)
    - Choose an executor (e.g., `tokio`).
    - Configure transport: TCP (`libp2p-tcp`).
    - **Integrate security protocol: Noise.** (`libp2p-noise`). (DEVELOPMENT_FLOW.md Phase 8)
    - **Integrate stream multiplexer: Yamux.** (`libp2p-yamux` or mplex). (DEVELOPMENT_FLOW.md Phase 8)
- [ ] Define `NetworkService` (or similar) struct to manage the Swarm and related logic.
- [ ] Implement basic Swarm event loop and event handling.

## II. Peer Discovery

- [ ] Implement mDNS for local peer discovery.
    - Crate: `libp2p-mdns`.
- [ ] Implement Kademlia DHT for wider peer discovery (more advanced, can be deferred if mDNS is sufficient for initial local testing).
    - Crate: `libp2p-kad`.
- [ ] Support for bootstrap node list for initial connections.
- [ ] Integrate `libp2p-identify` protocol for exchanging peer information.

## III. Message Definition and Serialization

- [ ] Define core network message enums/structs (shared between protocols):
    - **`TxMessage`**: Contains a `Transaction`. (DEVELOPMENT_FLOW.md Phase 8)
    - **`BlockMessage`**: Contains a `Block`. (DEVELOPMENT_FLOW.md Phase 8)
    - **`SyncRequest`**: E.g., `GetHeadersRequest { from_height: BlockHeight, count: u32 }`, `GetBlocksRequest { hashes: Vec<Hash> }`. (DEVELOPMENT_FLOW.md Phase 8)
    - **`SyncResponse`**: E.g., `HeadersResponse { headers: Vec<BlockHeader> }`, `BlocksResponse { blocks: Vec<Block> }`. (DEVELOPMENT_FLOW.md Phase 8)
- [ ] Implement `Serialize` and `Deserialize` for all message types (using `bincode`).

## IV. Communication Protocols

### A. Gossipsub for Broadcasting Transactions and Blocks

- [ ] **Setup `libp2p-gossipsub`.** (DEVELOPMENT_FLOW.md Phase 8)
    - Configure message authenticity (e.g., signed by origin or anonymous).
    - Define Gossipsub topics (e.g., `blocks_topic`, `transactions_topic`).
- [ ] **Implement broadcasting of new transactions via Gossipsub.** (DEVELOPMENT_FLOW.md Phase 8)
    - When a new transaction is received locally (e.g. from CLI or another peer), publish it.
- [ ] **Implement broadcasting of new blocks via Gossipsub.** (DEVELOPMENT_FLOW.md Phase 8)
    - When a new block is mined/created, publish it.
- [ ] Handle incoming Gossipsub messages:
    - Deserialize messages (`TxMessage`, `BlockMessage`).
    - **Deduplicate messages by hash.** (DEVELOPMENT_FLOW.md Phase 8)
        - Maintain a short-lived cache of seen message hashes.
    - Pass valid, new messages to relevant modules (Mempool, Consensus/Block Handler).
- [ ] ✅ **Milestone Check (Partial):** Transactions and blocks can be broadcast and received by peers. (Part of DEVELOPMENT_FLOW.md Phase 8 Milestone: "Transaction sent by Node A is received by Node B" specifically for Tx gossiping).

### B. Request-Response for Chain Synchronization

- [ ] Setup `libp2p-request-response` protocol.
    - Define request and response types (`SyncRequest`, `SyncResponse`).
- [ ] Implement handler logic for incoming `SyncRequest` messages:
    - Fetch requested data (headers, blocks) from Storage Layer.
    - Send `SyncResponse` back to the requester.
- [ ] Implement client logic to send `SyncRequest` messages to peers and handle `SyncResponse`.
    - This will be used by the chain synchronization logic (Phase 10).

## V. Network Manager / Event Handling

- [ ] Create a central networking component/task that:
    - Spawns the libp2p Swarm.
    - Listens on configured address(es).
    - Dials bootstrap/known peers.
    - Receives events from the Swarm (new peers, incoming messages, etc.).
    - Provides an API for other node components to send messages (e.g., `broadcast_transaction`, `broadcast_block`, `request_headers_from_peer`).
    - Emits events/callbacks to other modules when network messages are received (e.g., `NewTransactionReceived`, `NewBlockReceived`).

## VI. Error Handling & Logging

- [ ] Define `NetworkError` enum.
- [ ] Implement comprehensive `tracing` logging for P2P events, connections, disconnections, messages sent/received, errors.

## VII. Unit & Integration Tests

- [ ] Test PeerId generation and loading.
- [ ] Test message (de)serialization.
- [ ] Test Gossipsub: broadcast and receive a message between two local test nodes.
    - ✅ **Milestone Check:** A `TxMessage` sent by one local test node is successfully received and deserialized by another via Gossipsub. (Covers DEVELOPMENT_FLOW.md Phase 8 Milestone: "Transaction sent by Node A is received by Node B").
- [ ] Test Request-Response: one node requests data, the other responds successfully.
- [ ] Test peer discovery (mDNS locally).

## VIII. Configuration

- [ ] Allow configuration of listen address, bootstrap nodes, peer key path. 