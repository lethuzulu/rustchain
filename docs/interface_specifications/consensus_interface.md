## Consensus Engine Interface

**Purpose:** Manages block proposal eligibility, validation of blocks according to consensus rules (e.g., PoS signature, proposer turn), fork choice, and providing information about the current validator set or schedule.

**Public Functions/Methods:**

*   **`process_incoming_block(block: &Block, current_chain_state: &dyn ChainView) -> Result<ConsensusOutcome, ConsensusError>`**
    *   **Description:** Validates an incoming block against consensus rules (e.g., correct proposer, valid signature, correct chain linkage, adherence to fork choice rules). This is a critical step before a block can be applied to the state machine or saved to storage.
    *   **Parameters:**
        *   `block`: The block received from the network or proposed locally.
        *   `current_chain_state`: A read-only view of the current blockchain state, needed for context (e.g., current height, tip hash, validator set for the block's height).
    *   **Returns:** `ConsensusOutcome` indicating if the block is valid, part of a fork, etc., or `ConsensusError` if validation fails.
    *   **Note:** This function implies internal stages like structural validation (hashes, Merkle root, format - potentially delegated or assumed done) and semantic/stateful consensus validation (proposer signature, validator turn, fork choice logic).

*   **`get_validator_for_height(height: u64, current_chain_state: &dyn ChainView) -> Result<Address, ConsensusError>`**
    *   **Description:** Determines the expected validator (proposer) for a given block height based on the current chain state and validator set.
    *   **Parameters:** `height`, `current_chain_state`.
    *   **Returns:** The `Address` of the expected validator, or `ConsensusError` (e.g., if height is invalid or validator set cannot be determined).

*   **`select_next_validator(current_height: u64, current_chain_state: &dyn ChainView) -> Result<Address, ConsensusError>`**
    *   **Description:** Selects the validator responsible for proposing the *next* block (i.e., for `current_height + 1`).
    *   **Parameters:** `current_height`, `current_chain_state`.
    *   **Returns:** The `Address` of the next validator, or `ConsensusError`.

*   **`resolve_fork(current_tip_hash: Hash, new_competing_block_hash: Hash, chain_view: &dyn ChainView) -> Result<Hash, ConsensusError>`**
    *   **Description:** Implements the fork choice rule (e.g., longest chain, GHOST) to determine the canonical chain tip when a fork is detected.
    *   **Parameters:**
        *   `current_tip_hash`: The hash of the current recognized chain tip.
        *   `new_competing_block_hash`: The hash of a new block that could represent an alternative chain tip.
        *   `chain_view`: A read-only view to access necessary block/header information (like height, parent hashes).
    *   **Returns:** The `Hash` of the block that should be considered the new canonical chain tip, or `ConsensusError`.

*   **`request_block_production_trigger(chain_view: &dyn ChainView) -> Result<Option<BlockProductionInfo>, ConsensusError>`**
    *   **Description:** Called by the node orchestrator, typically on a timer or when new transactions arrive, to check if this node is the current proposer and should attempt to produce a block.
    *   **Parameters:** `chain_view` to know the current chain state (height, tip, validator schedule).
    *   **Returns:** `Ok(Some(BlockProductionInfo))` if this node should produce a block (containing necessary info like parent hash, height), `Ok(None)` if not its turn, or `ConsensusError`.

**Events Emitted (Conceptual - could be direct returns or via a registered event bus):**

*   `BlockAccepted(block_hash: Hash, block_height: u64)`
*   `BlockRejected(block_hash: Hash, reason: ConsensusError)`
*   `ForkDetected(common_ancestor_hash: Hash, new_chain_tip_hash: Hash)`
*   `NewChainTip(block_hash: Hash, block_height: u64)`

**Data Structures:**

*   `Block`, `BlockHeader`, `Address`, `Hash` (see `../architecture/data_structures.md`)
*   `Validator` (Could be an `Address` or a struct with more info like stake, defined in `../architecture/data_structures.md` or here)
*   `ChainView` (A trait defining read-only access to blockchain data relevant for consensus, e.g., `get_header(hash)`, `get_height()`, `get_validator_set_for_epoch(epoch)`).
*   `ConsensusOutcome` (e.g., `enum { ValidBlock, IgnoredStale, ForkBranch }`)
*   `BlockProductionInfo` (e.g., struct containing `parent_hash`, `block_number_to_produce`, `timestamp_slot`)

**Error Handling:**

*   `enum ConsensusError {`
    *   `InvalidSignature,`
    *   `InvalidProposer,`         // Block signed by unexpected validator
    *   `MismatchedParentHash,`
    *   `BlockHeightOutOfOrder,`
    *   `FutureBlock,`             // Block timestamp too far in the future
    *   `StaleBlock,`              // Block too old or already processed
    *   `ForkChoiceFailed(String),`
    *   `ValidatorSetUnavailable,`
    *   `InternalError(String)`
    *   `}` 