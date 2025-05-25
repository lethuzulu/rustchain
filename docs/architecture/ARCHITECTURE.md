# 🧠 RustChain Architecture Overview

RustChain is a minimalist Proof-of-Stake Layer 1 blockchain written in Rust. It is designed to prioritize modularity, clarity, and pedagogical value for blockchain engineers and researchers.

*Note: Detailed development phases and actionable task breakdowns are tracked in [docs/development_breakdown/ROADMAP.md](../development_breakdown/ROADMAP.md). This document and its subpages focus on architecture and design, not implementation planning.*

This document gives a high-level overview of the system and links to the full architecture breakdown.

For the overall project goals, development roadmap, and component breakdown details, please refer to the [PROJECT_PLAN.md](../PROJECT_PLAN.md).

## 📌 Component Modules

- [Node Lifecycle](./node_lifecycle.md)
- [Networking Layer (libp2p)](./networking.md)
- [Consensus Protocol](./consensus.md)
- [Transaction & Block Format](./data_structures.md)
- [State Machine](./state_machine.md)
- [Storage Layer](./storage_layer.md)
- [Wallet Interface](./wallet.md)
- [Validator Node Responsibilities](./validator_node.md)
- [Block Explorer (Optional)](./block_explorer.md)

## 📊 Architecture Diagram

*Diagram placeholder — to be inserted*

## 🔁 Transaction Flow Summary

1. Wallet signs and submits transaction to a node.
2. Node validates transaction and stores it in the mempool.
3. Proposer is selected via PoS and creates a new block.
4. Block is signed, gossiped to peers, and validated.
5. Nodes apply block → update state → persist to DB. 