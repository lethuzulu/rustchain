# ⚖️ Consensus Engine (Static PoS): Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 5: Consensus Engine (Static PoS)](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-5-consensus-engine--static-pos)
- Integration with [Phase 9: Block Production Integration](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-9-block-production-integration) and [Phase 11: Genesis File & Dev Fixtures](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-11-genesis-file--dev-fixtures)

**Likely Crates/Tools:**
- `serde` (for loading validator set from genesis/config)
- `ed25519-dalek` or `secp256k1` (for signature verification of blocks)
- `sha2` (for hashing, if needed for any consensus specific data beyond block hash)
- `tracing` (for logging consensus decisions and errors)

This document lists the actionable development tasks for implementing RustChain's minimal Proof-of-Stake consensus engine.

## I. Module Definition and Configuration

- [ ] **Define `consensus.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] Define `ConsensusConfig` struct (e.g., validator set, epoch length if applicable - for minimal, static set is fine).
- [ ] Define `ConsensusEngine` struct to encapsulate consensus logic.
    - It will need access to the current chain state (e.g., current block height, validator set).
- [ ] Implement `new(config: ConsensusConfig, initial_validators: Vec<PublicKey>) -> Self` constructor.
    - **Validator Set:** (DEVELOPMENT_FLOW.md Phase 5) Load/initialize a static list of validator public keys (e.g., from `ConsensusConfig` which might get it from a genesis file representation later).

## II. Proposer Selection

- [ ] **Implement round-robin proposer selection logic.** (DEVELOPMENT_FLOW.md Phase 5)
    - Function signature: `get_proposer(block_height: BlockHeight, current_timestamp: Timestamp) -> Result<PublicKey, ConsensusError>`.
    - Logic: Based on block height (or slot/round derived from height/time) and the static validator set.
    - Ensure deterministic selection.

## III. Block Validation (Consensus Rules)

- [ ] **Implement block proposer validation.** (DEVELOPMENT_FLOW.md Phase 5)
    - Function signature: `validate_block_proposer(block_header: &BlockHeader, chain_context: &ChainContext) -> Result<(), ConsensusError>`.
    - Check if the `proposer` field in the `BlockHeader` matches the expected proposer for that height/slot.
- [ ] Implement block signature verification (using the proposer's public key).
    - This might re-use logic from `block.rs` but is a key consensus check.
- [ ] Implement basic block header validation rules (beyond what `Block` struct enforces):
    - Timestamp validation (e.g., not too far in past/future, after parent).
    - Parent hash consistency (part of chain linkage).
- [ ] (Future/Optional for minimal PoS): Implement rules against specific attacks if applicable (e.g., simple forms of equivocation if context allows detection).

## IV. Fork Choice Rule

- [ ] **Implement the longest-chain fork choice rule.** (DEVELOPMENT_FLOW.md Phase 5)
    - Function signature: `select_best_chain(current_tip: &BlockHeader, new_candidate_tip: &BlockHeader, chain_store: &dyn ChainQuery) -> Result<ChainPreference, ConsensusError>`.
    - `ChainQuery` would be a trait to abstract access to block information (heights, parent hashes).
    - `ChainPreference` could be an enum like `PreferCurrent`, `PreferCandidate`.
    - Logic: Compare block heights; if equal, could use a tie-breaking rule (e.g., first seen, or smallest hash - though simple height is fine for minimal).

## V. Block Production (Consensus Aspects)

- [ ] Define logic for a validator to create a new block when it's their turn:
    - This involves using the `ConsensusEngine` to know if it *can* propose.
    - The actual block building (selecting transactions from mempool, creating Merkle root) is in `block.rs` and orchestrated by the validator node.
    - This task is about the consensus decision to *initiate* block creation.
    - Function: `can_propose_block(my_public_key: &PublicKey, current_block_height: BlockHeight, current_timestamp: Timestamp) -> bool`.

## VI. Error Handling

- [ ] Define `ConsensusError` enum.
    - E.g., `InvalidProposer`, `InvalidSignature`, `InvalidTimestamp`, `NotInValidatorSet`, `ForkChoiceError`.

## VII. Unit Tests

- [ ] Test proposer selection logic for various heights with a known validator set.
- [ ] Test block proposer validation (correct and incorrect proposers).
- [ ] Test longest-chain fork choice rule with different chain scenarios.
- [ ] Test `can_propose_block` logic.
- [ ] ✅ **Milestone Check (Logic part):** The consensus engine can correctly determine proposers, validate incoming blocks based on PoS rules, and apply the fork choice rule. (Corresponds to DEVELOPMENT_FLOW.md Phase 5 Milestone: "Validator node correctly produces and accepts blocks" - this covers the rule-based acceptance part. Full production/acceptance involves node integration).

## VIII. Logging

- [ ] Add `tracing` logs for key consensus events: proposer selection, block validation (pass/fail with reasons), fork choice decisions. 