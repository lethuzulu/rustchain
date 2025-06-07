# 🌐 Networking Layer

RustChain uses libp2p to form a fully decentralized peer-to-peer network **for inter-node communication (e.g., block and transaction propagation, peer discovery, and chain synchronization).**

**Note:** Communication between a CLI wallet and a node is handled via a separate JSON-RPC interface provided by the node, not directly over libp2p in the minimal design.

## 🧩 Peer Discovery

- Static bootstrap peer list (via config)
- Local discovery via `libp2p::mdns` in dev mode

## 📦 Message Types

| Message Type     | Description                        |
|------------------|------------------------------------|
| `TxMessage`      | Carries a single signed transaction|
| `BlockMessage`   | Carries a proposed block           |
| `SyncRequest`    | Asks for headers or blocks         |
| `SyncResponse`   | Sends block data                   |
| `Ping`           | Keep-alive                         |

**Serialization:** Message payloads (e.g., `TxMessage`, `BlockMessage`) are serialized using `bincode` before transmission.

## 🔄 Propagation Strategy

- Uses `libp2p::gossipsub` for transaction and block gossip
- Deduplicated via hash

## 🔒 Transport Stack

| Layer          | Implementation         |
|----------------|------------------------|
| Encryption     | `libp2p::noise`        |
| Multiplexing   | `yamux`                |
| Protocol       | Gossipsub + req-resp   |
| Peer Identity  | libp2p keypair-derived |
