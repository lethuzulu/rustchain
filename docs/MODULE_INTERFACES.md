# Module Interface Specifications (Overview)

This document provides an overview and index for the detailed interface specifications of RustChain's core modules. Clear interfaces are crucial for modularity, testability, and ensuring a clean separation of concerns between different parts of the blockchain system.

## General Principles

- **Minimal & Complete:** Interfaces should expose only necessary functionality, but enough to allow modules to perform their roles.
- **Contract-Based Interaction:** Modules should interact primarily through these defined public interfaces, treating them as contracts.
- **Explicit Error Handling:** Errors should be clearly defined and returned, allowing callers to handle failures gracefully.
- **Defined Data Structures:** Data structures passed across interfaces should be clearly defined (often referencing `docs/architecture/data_structures.md`).
- **Loose Coupling:** Aim for interfaces that reduce tight dependencies between modules, allowing for independent development and evolution.

---

## Core Module Interfaces

Below are links to the detailed interface specifications for each major RustChain component:

1.  **[Networking Layer Interface](./interface_specifications/networking_interface.md)**
    *   Handles all peer-to-peer communication, including message broadcasting, peer discovery, and connection management.

2.  **[Storage Layer Interface](./interface_specifications/storage_interface.md)**
    *   Manages the persistent storage of blockchain data, including blocks, headers, state, and metadata.

3.  **[Consensus Engine Interface](./interface_specifications/consensus_interface.md)**
    *   Manages block proposal, validation according to consensus rules, fork choice, and finality.

4.  **[Mempool Interface](./interface_specifications/mempool_interface.md)**
    *   Manages a pool of unconfirmed transactions, providing transactions for block creation and ensuring transaction validity.

5.  **[State Machine Interface](./interface_specifications/state_machine_interface.md)**
    *   Defines and enforces the rules for state transitions based on transactions; validates and applies transactions to the world state.

6.  **[Wallet RPC Interface (Node-Side)](./interface_specifications/wallet_rpc_interface.md)**
    *   Defines the JSON-RPC endpoints the node exposes for wallet interactions (e.g., `get_balance`, `submit_transaction`).

---

*This overview and the linked detailed specifications are living documents and will evolve with the project.* 