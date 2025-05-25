# 🌀 Node Lifecycle

This document describes the lifecycle of a validator node from startup to shutdown.

## 🚀 Startup Sequence

1. **Configuration Load**: Load from CLI or file (network keys, validator signing key, peer list, db path).
2. **Keypair Setup**: 
    - Load or generate node identity keypair (for libp2p `PeerId`).
    - Load validator signing keypair (for block signing, if node is a validator).
3. **State Load**: Open RocksDB and retrieve chain tip and current state.
4. **P2P Init**: Set up `libp2p::Swarm` with the defined networking stack (e.g., TCP, Noise, Yamux, Gossipsub for message propagation, Request-Response for sync).
5. **Subsystems Start**:
    - Mempool (in-memory)
    - State manager
    - Consensus engine (async task)
6. **Enter Main Event Loop**:
    - Polls for:
        - P2P messages
        - Timer ticks
        - Internal tx/block events

## 🛑 Shutdown

- Close DB
- Save any mempool snapshot (future)
- Disconnect peers
