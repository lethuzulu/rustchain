# 🧪 Testing Strategy and Tasks

**Owner:** TBD (Lead/All Developers)
**Status:** Ongoing throughout development, with focused phases for hardening and comprehensive integration.

**Relevant Development Flow Phases:**
- Unit tests are integrated into each component's development (see individual `_tasks.md` files).
- [Phase 2: Transaction Structs & Validation Logic](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-2-transaction-structs--validation-logic) (early unit testing focus)
- [Phase 13: Manual Testing & Hardening](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-13-manual-testing--hardening)
- [Phase 14: Unit & Integration Tests](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-14-unit--integration-tests)

This document outlines the overall testing strategy for RustChain and lists tasks related to manual testing, hardening, and comprehensive integration testing.

## I. Testing Philosophy

RustChain development will follow a multi-layered testing approach:

1.  **Unit Tests:** Each module and significant function should have unit tests covering its specific logic, including happy paths and edge cases. These are defined within each component's `_tasks.md` file.
    - Refer to sections like "Unit Tests" or "Testing" in:
        - `core_data_structures_tasks.md`
        - `cli_wallet_tasks.md`
        - `mempool_tasks.md`
        - `consensus_engine_tasks.md`
        - `state_machine_tasks.md`
        - `storage_layer_tasks.md`
        - `p2p_networking_tasks.md`
2.  **Integration Tests:** These tests verify the interaction between multiple components. Key integration tests are defined in `validator_node_tasks.md` (e.g., node lifecycle, transaction flow, basic sync).
3.  **Manual Testing & Hardening (Phase 13):** Focused effort to manually explore system behavior, especially around failure modes and complex interactions.
4.  **End-to-End (E2E) Tests:** Automated tests simulating real user scenarios from wallet to chain state changes across multiple nodes.

## II. Manual Testing & Hardening Tasks (Phase 13)

These tasks are typically performed on a local testnet with multiple nodes.

- [ ] **Set up a local multi-node testnet environment.**
    - E.g., 3-5 validator nodes.
    - Script or document easy setup/teardown.
- [ ] **Manually test invalid transaction submissions:** (DEVELOPMENT_FLOW.md Phase 13)
    - Insufficient balance.
    - Incorrect nonce (too low, too high).
    - Invalid signature.
    - Malformed transaction data.
    - Sending to non-existent address (verify account creation or specific handling).
- [ ] **Manually test fork behavior and resolution:** (DEVELOPMENT_FLOW.md Phase 13)
    - Simulate network partitions and observe chain divergence.
    - Heal partition and observe if longest chain rule is correctly applied and nodes re-converge.
    - Test scenarios with blocks arriving out of order.
- [ ] **Test edge cases for block production and validation:** (DEVELOPMENT_FLOW.md Phase 13)
    - **Double spend attempts:** Submit conflicting transactions to different nodes/mempools.
    - **Invalid proposer:** Manually craft or force a block from an incorrect proposer.
    - **Invalid block signature:** Tamper with a block signature.
    - Blocks with max transactions/size.
    - Blocks with no transactions.
- [ ] **Test node robustness:**
    - Graceful shutdown and restart of nodes; verify state recovery.
    - Abrupt shutdown (kill -9) and restart; assess data integrity (expect potential minor loss if not fsyncing aggressively, but DB should not corrupt).
    - Test with slow/lossy network conditions (if tools allow simulation).
- [ ] **Improve logs and error messages based on observations.** (DEVELOPMENT_FLOW.md Phase 13)
    - Ensure errors are clear, actionable, and provide sufficient context.
- [ ] **Document any found issues, unexpected behaviors, or areas for improvement.**
- [ ] ✅ **Milestone Check (Hardening):** Local testnet reliably handles common failure cases and edge conditions; identified issues are documented or fixed. (Corresponds to DEVELOPMENT_FLOW.md Phase 13 Milestone).

## III. Comprehensive Unit & Integration Tests (Phase 14)

This phase ensures broad test coverage and fills any gaps.

- [ ] **Review unit test coverage for all core modules.**
    - `transaction.rs` (DEVELOPMENT_FLOW.md Phase 14)
    - `block.rs` (hashing, Merkle root) (DEVELOPMENT_FLOW.md Phase 14)
    - `state_machine.rs` (state transitions)
    - `consensus.rs` (proposer logic, validation rules)
    - `mempool.rs` (add, remove, get)
    - `storage.rs` (put, get, atomic writes)
    - Aim for high coverage of critical logic paths.
- [ ] **Expand integration tests for node interactions:**
    - **Syncing scenarios:** (DEVELOPMENT_FLOW.md Phase 14)
        - New node joining an active network.
        - Node rejoining after being offline and syncing missed blocks.
        - Syncing with conflicting (short-lived) forks.
    - **End-to-end wallet → transaction → block flow:** (DEVELOPMENT_FLOW.md Phase 14)
        - Multiple transactions from multiple wallets being included in blocks across multiple nodes.
        - Verify state changes (balances, nonces) are consistent across all synced nodes.
    - Test validator rotation if dynamic validators were implemented (not in minimal PoS, but for future).
    - Test behavior with maximum number of connected peers.
- [ ] **(Optional) Implement property-based testing for critical data structures or state transitions.**
    - E.g., using `proptest`.
- [ ] **(Optional) Setup automated nightly/CI test runs for longer E2E scenarios.**
- [ ] ✅ **Milestone Check (Comprehensive Tests):** A full suite of unit and integration tests pass, covering core logic, component interactions, and key E2E scenarios like syncing and state transitions. (Corresponds to DEVELOPMENT_FLOW.md Phase 14 Milestone).

## IV. Test Data and Fixtures

- [ ] Create or document how to generate test transactions, blocks, and genesis files for various scenarios. (Partially covered by DEVELOPMENT_FLOW.md Phase 11 for dev fixtures).
- [ ] Ensure test data is easily usable in automated tests. 