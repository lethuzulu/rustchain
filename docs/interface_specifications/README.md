# Module Interface Specifications (Overview)

This document provides an overview and index for the detailed interface specifications of RustChain's core modules. Clear interfaces are crucial for modularity, testability, and ensuring a clean separation of concerns between different parts of the blockchain system.

## General Principles

- **Minimal & Complete:** Interfaces should expose only necessary functionality, but enough to allow modules to perform their roles.
- **Contract-Based Interaction:** Modules should interact primarily through these defined public interfaces, treating them as contracts.
- **Explicit Error Handling:** Errors should be clearly defined and returned, allowing callers to handle failures gracefully. (See specific `...Error` enums in each interface file).
- **Defined Data Structures:** Data structures passed across interfaces should be clearly defined (often referencing `../architecture/data_structures.md`).
- **Loose Coupling:** Aim for interfaces that reduce tight dependencies between modules, allowing for independent development and evolution.

---

## Core Module Interfaces

Below are links to the detailed interface specifications for each major RustChain component:

1.  **[Networking Layer Interface](./networking_interface.md)**
    *   Handles all peer-to-peer communication, including message broadcasting, peer discovery, and connection management.

2.  **[Storage Layer Interface](./storage_interface.md)**
    *   Manages the persistent storage of blockchain data, including blocks, headers, state, and metadata.

3.  **[Consensus Engine Interface](./consensus_interface.md)**
    *   Manages block proposal, validation according to consensus rules, fork choice, and finality.

4.  **[Mempool Interface](./mempool_interface.md)**
    *   Manages a pool of unconfirmed transactions, providing transactions for block creation and ensuring transaction validity.

5.  **[State Machine Interface](./state_machine_interface.md)**
    *   Defines and enforces the rules for state transitions based on transactions; validates and applies transactions to the world state.

6.  **[Wallet RPC Interface (Node-Side)](./wallet_rpc_interface.md)**
    *   Defines the JSON-RPC endpoints the node exposes for wallet interactions (e.g., `get_balance`, `submit_transaction`).

7.  **[Wallet Library Interface](./wallet_library_interface.md)**
    *   Defines a standard library interface for core wallet operations like key management and transaction creation, intended for use by CLI tools or other client applications.

---

## Node Orchestration Overview

The main validator node software is responsible for initializing and coordinating the various modules described above. Its primary entry point is typically a `run_node(config: NodeConfig) -> Result<(), NodeError>` function.

Detailed architectural descriptions of the node's lifecycle and internal workings can be found in:
*   `../architecture/validator_node.md`
*   `../architecture/node_lifecycle.md`

The node utilizes the defined interfaces of each module to perform its tasks (e.g., receiving messages via the Networking Layer interface, proposing blocks using the Consensus Engine interface, storing data through the Storage Layer interface).

---

*This overview and the linked detailed specifications are living documents and will evolve with the project.* 