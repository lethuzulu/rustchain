# Rustchain: Minimal Layer 1 Blockchain - Project Plan

## 1. Overall Architecture

This section will describe the high-level interaction between the major components of the Rustchain blockchain.

A visual diagram will be added below to illustrate these interactions.

*   **Networking Layer (libp2p):** Responsible for peer discovery, message propagation (transactions, blocks), and maintaining network connectivity. Nodes will use libp2p to form a peer-to-peer network.
*   **Transaction Pool (Mempool):** A temporary holding area for unconfirmed transactions received from the network. Candidate block producers (validators chosen by consensus) will pull transactions from the mempool to assemble into a proposed block.
*   **Block Construction & Validation:** Validators selected by the consensus mechanism are responsible for constructing new blocks. This involves selecting transactions from the mempool, ordering them, and creating a block structure. This proposed block is then broadcast for validation.
*   **Consensus Layer (Proof-of-Stake):** Ensures agreement among validators on the next valid block to be added to the chain. It involves proposing blocks (often by a selected leader), validating proposed blocks according to network rules (including transaction validity via the State Machine), and running the chosen Proof-of-Stake algorithm to achieve finality.
*   **State Machine:** Defines the rules for how transactions update the blockchain's state (e.g., account balances, contract storage). The state itself is persisted by the Storage Layer. It's crucial for transaction validation within the consensus process.
*   **Storage Layer:** Manages the persistent storage of the blockchain data, including all validated blocks and the current world state (e.g., account balances, staked amounts).
*   **Blockchain Core:** Manages the chain itself – applying validated blocks, handling the sequence of blocks, and maintaining the overall integrity of the ledger.
*   **CLI Wallet:** Allows users to generate keys, create, sign, and broadcast transactions to the network. It interacts with a node's P2P interface or a dedicated RPC endpoint.
*   **Validator Node Software:** The application run by validators. It bundles all the above components: networking, transaction pool interaction, block construction, consensus participation, state management, and storage interaction.
*   **(Optional) Block Explorer:** A separate application (or a feature of the node) that reads blockchain data from the Storage Layer and presents it in a user-friendly way (e.g., web UI or CLI).

*(Diagram to be added here)*

## 2. Component Breakdown

*(To be detailed further)*

*   **P2P Network (libp2p)**
    *   **Key Functionalities (Minimal):**
        *   **Peer Identity:** Each node has a unique `PeerId` (cryptographic public key).
        *   **Transport:** TCP for sending bytes between nodes.
        *   **Security:** Encrypted and authenticated communication using `noise` protocol.
        *   **Stream Multiplexing:** Multiple logical streams over a single connection (e.g., `yamux` or `mplex`).
        *   **Peer Discovery:**
            *   Bootstrap Nodes: A list of known nodes for initial connection.
            *   mDNS: Local network peer discovery for testing/development.
        *   **Message Propagation (Gossip):** Use `libp2p-gossipsub` to broadcast transactions and blocks on defined topics (e.g., "new_transactions", "new_blocks").
        *   **Block/Header Synchronization:** Allow nodes to request missing blocks/headers to catch up (e.g., via a request-response protocol).
        *   **Structured Internal Modules:** Clear separation of concerns within the networking code (see Design Decisions).
    *   **Design Decisions (Focusing on Minimal `libp2p`):**
        *   Utilize `Swarm` to manage connections and a custom `NetworkBehaviour` to combine `libp2p` components.
        *   **Internal Module Structure (Conceptual):**
            *   `IdentityService` (or similar): Manages the node's own `PeerId` and related cryptographic keys for networking.
            *   `PeerManager`: Wraps `Swarm` functionalities for connection and peer lifecycle management (dialing, disconnects, peer reputation/banning).
            *   `GossipEngine`: Manages `libp2p-gossipsub` for transaction and block propagation.
            *   `SyncProtocol`: Implements a request-response protocol (e.g., using `libp2p-request-response`) for block/header synchronization.
        *   Initial Discovery: Bootstrap node list (configurable/hardcoded) + mDNS.
        *   Transport: TCP.
        *   Security Protocol: `noise`.
        *   Multiplexer: `yamux` (preferred) or `mplex`.
        *   Message Propagation: `libp2p-gossipsub`.
        *   Message Content/Serialization: Define Rust structs for transaction/block messages; serialize with `bincode` (for Rust-to-Rust simplicity) or `prost` (if using Protobuf definitions).
    *   **Challenges/Learning:**
        *   Understanding `libp2p` `Swarm` and `NetworkBehaviour` composition.
        *   Configuring and integrating chosen `libp2p` modules (Transport, Security, Multiplexing, Discovery, Gossipsub).
        *   Defining `gossipsub` topics and managing publish/subscribe logic.
        *   Integrating P2P layer with other blockchain components (transaction submission, block production).
        *   Handling asynchronous events from `libp2p` using Rust's `async/await` (e.g., with `tokio`).
*   **Block Structure (Minimal)**
    *   **Key Functionalities:**
        *   **Chain Linking:** Securely link to the previous block using its hash.
        *   **Transaction Grouping:** Bundle an ordered list of transactions.
        *   **Integrity & Verifiability:** Use a Merkle root of transactions for integrity.
        *   **Metadata Provision:** Include block number, timestamp, and validator identity.
        *   **Uniqueness & Authenticity:** Each block has a unique hash, and the header is signed by the validator.
    *   **Design Decisions (Minimal):**
        *   **Overall Block Composition:** `BlockHeader` and `BlockBody`.
        *   **`BlockHeader` Fields (Minimal):**
            *   `parent_hash`: Hash of the previous block's header (e.g., `H256` - a fixed-size 32-byte hash).
            *   `block_number`: Sequential block height (e.g., `u64`).
            *   `timestamp`: Unix timestamp (seconds since epoch) (e.g., `u64`).
            *   `transactions_root`: Merkle root hash of transactions in the `BlockBody`.
            *   `validator_address`: Public address of the block's proposer/validator.
            *   `validator_signature`: Signature from the `validator_address` over the hash of (parent_hash, block_number, timestamp, transactions_root, validator_address).
            *   *(Deferred: `state_root`)*
        *   **`BlockBody` Fields (Minimal):**
            *   `transactions`: An ordered list of `Transaction` objects.
        *   **`Transaction` Structure (Minimal - to be detailed separately):**
            *   Assume a simple transfer transaction for now: `sender`, `recipient`, `amount`, `nonce`, `signature`.
            *   Staking-related transactions can be designed later if they are distinct on-chain transaction types.
        *   **Hashing Algorithm:** SHA-256 for block hashes, Merkle roots, and transaction hashes.
        *   **Serialization Format:** `bincode` for efficient Rust-to-Rust serialization.
    *   **Challenges/Learning:**
        *   Correct implementation of Merkle tree generation and verification.
        *   Defining the canonical serialization of the header fields that are input to the validator's signature and the block hash calculation.
        *   Ensuring the transaction signing and verification process is robust.
        *   Managing fixed-size hash types and addresses.
*   **Consensus (Proof-of-Stake) (Minimal)**
    *   **Key Functionalities:**
        *   **Validator Set Management (Static):** Maintain a predefined list of active validators (defined at genesis).
        *   **Block Proposer Selection (Round-Robin):** Deterministically choose a validator from the static set to propose the next block for a given slot/turn (e.g., based on `block_number % num_validators`).
        *   **Block Validation (Consensus Rules):** Verify that a proposed block is from the legitimate proposer for the current turn and is correctly signed.
        *   **Chain Selection (Fork Choice - Longest Chain):** In case of forks, nodes favor the valid chain with the greatest block height. A deterministic tie-breaker (e.g., lowest block hash) can be used if necessary.
    *   **Design Decisions (Minimal):**
        *   **Validator Set:** Fixed at genesis; no on-chain staking/unstaking for this version. Stake amounts are effectively equal or not tracked for consensus weight beyond set membership.
        *   **Block Proposer Selection:** Round-robin scheduling based on the genesis validator list and current block number.
        *   **Block Validation (PoS Specific):**
            *   `BlockHeader.validator_address` must match the expected proposer for the current `block_number`.
            *   `BlockHeader.validator_signature` must be a valid signature from this address over the canonical hash of specified header fields.
        *   **Chain Selection/Fork Choice:** Longest valid chain rule. No BFT-style voting or finality gadgets.
        *   **Rewards and Slashing:** None for the minimal version (no on-chain economic incentives/penalties).
    *   **Challenges/Learning:**
        *   Implementing deterministic round-robin proposer selection across all nodes.
        *   Secure handling of validator keys for block signing.
        *   Integrating consensus logic with networking, block validation, and the blockchain core.
        *   Defining behavior for missed proposer slots (e.g., timeout, or next validator proceeds after observing a missed slot – initially might assume validators are online).
        *   Precisely defining the canonical form of header data to be signed.
*   **CLI Wallet (Minimal)**
    *   **Key Functionalities:**
        *   **Key Pair Generation:** Generate new Ed25519 public/private key pairs.
        *   **Key Storage (Simple):** Store private keys locally in a simple file format (e.g., raw bytes). *Security Note: For demo purposes only; not secure for real assets.*
        *   **Address Display:** Show user their public address.
        *   **Balance Inquiry:** Query account balance from a node (requires basic node query mechanism).
        *   **Nonce Inquiry:** Query current account nonce from a node (requires basic node query mechanism).
        *   **Transaction Creation:** Construct a basic transfer transaction (sender, recipient, amount, nonce).
        *   **Transaction Signing:** Sign the transaction using the user's stored private key.
        *   **Transaction Submission:** Send the signed transaction to a connected node.
    *   **Design Decisions (Minimal):**
        *   **Command Structure:** Subcommands like `generate-keys`, `show-address`, `get-balance <ADDRESS>`, `send-tx --to <RECIPIENT> --amount <AMOUNT> [--keyfile <PATH_TO_KEY>]`.
        *   **Key Management:** Private keys as raw byte files. No complex key derivation paths.
        *   **Nonce & Balance Acquisition:** Wallet queries a connected node for current nonce and balance before sending a transaction.
        *   **Node Interaction:** Connect to a configurable node address for transaction submission and queries. Implement a basic request-response protocol on the node for these queries.
        *   **Output Format:** Human-readable console output.
        *   **Cryptography Crates:** `ed25519-dalek`, `rand`.
    *   **Challenges/Learning:**
        *   Secure (even if simplified) local private key handling.
        *   Correct transaction signing and serialization.
        *   Designing and implementing the wallet-node communication protocol for queries and submissions.
        *   CLI argument parsing (e.g., `clap` crate).
        *   Local file management for keys.
*   **Validator Node Software (Minimal)**
    *   **Key Functionalities:**
        *   **Initialization:** Load configuration (node identity/keys, network settings, genesis validator list), initialize P2P Network, Blockchain (from storage), Mempool, Consensus, and Storage components.
        *   **P2P Network Participation:** Connect, discover peers, listen for/propagate transactions and blocks.
        *   **Transaction Handling:** Receive transactions from P2P, validate (syntax, signature, nonce), add to Mempool.
        *   **Block Production (when selected by Consensus):** Get notified by Consensus, select transactions from Mempool, construct new block (header, body, Merkle root, sign header), broadcast block via P2P.
        *   **Block Reception & Validation:** Receive blocks from P2P, pass to Consensus (PoS rules) and Blockchain core (full validation, state transitions).
        *   **Chain Management:** Add valid blocks, update world state, persist to Storage, handle basic fork resolution (longest chain).
        *   **State Queries (Basic Internal Service):** Provide mechanism for CLI Wallet to query balance/nonce.
        *   **Graceful Shutdown:** Save state and shut down cleanly.
    *   **Design Decisions (Minimal):**
        *   **Main Loop / Event-Driven Architecture:** Use `async/await` (e.g., `tokio`) for a main event loop processing events from P2P, internal timers, and internal query service.
        *   **Component Integration:** Define clear interfaces and interaction patterns between components.
        *   **Configuration:** Securely load validator private key, network settings, genesis info.
        *   **Concurrency:** Use `async` tasks for network I/O, message processing.
        *   **Error Handling:** Robust error handling and logging.
    *   **Challenges/Learning:**
        *   Orchestrating all components efficiently and non-blockingly.
        *   Consistent state management with concurrency.
        *   Implementing the main event loop and event dispatch.
        *   Secure validator private key management for block signing.
        *   Thorough integration testing.
        *   Designing and implementing the internal service for wallet queries.
*   **(Optional) Block Explorer (Minimal CLI Tool)**
    *   **Key Functionalities:**
        *   **View Blocks:** Display a list of blocks (e.g., latest N blocks).
        *   **View Block Details:** Show header information and list of transaction hashes for a given block (by number or hash).
        *   **View Transaction Details:** Show details for a given transaction hash.
        *   **Basic Navigation:** Query by block number/hash, transaction hash.
    *   **Design Decisions (Minimal):**
        *   **Type:** Command-Line Interface (CLI) tool.
        *   **Data Source:** Directly reads from the blockchain's data storage (e.g., RocksDB/Sled). No network interaction.
        *   **Functionality:** Read-only inspection tool.
        *   **Output:** Human-readable text to the console.
        *   **Commands (Examples):** `show-latest-blocks [COUNT]`, `get-block <NUMBER_OR_HASH>`, `get-tx <TX_HASH>`.
        *   **No Real-time Updates:** Queries state at command execution time.
    *   **Challenges/Learning:**
        *   Efficiently querying and deserializing data from the blockchain database.
        *   Clear and concise text-based data presentation.
        *   Can be a separate executable or a module of the main node software (e.g., invoked with specific flags like `rustchain-node explore ...`).

---

*For a detailed development roadmap, phased execution plan, and specific task breakdowns for each component, please refer to the documents within the `docs/development_breakdown/` directory, particularly `DEVELOPMENT_FLOW.md` and `ROADMAP.md`.* 