# 🔄 State Machine: Task Breakdown

**Owner:** TBD
**Status:** To Do

**Relevant Development Flow Phases:**
- [Phase 0: Project Scaffolding & Module Setup](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-0-project-scaffolding--module-setup)
- [Phase 2: Transaction Structs & Validation Logic](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-2-transaction-structs--validation-logic) (stateful validation part)
- [Phase 6: State Machine Execution](../../docs/development_breakdown/DEVELOPMENT_FLOW.md#phase-6-state-machine-execution)

**Likely Crates/Tools:**
- `serde` & `bincode` (for serializing Account/State if they are stored directly)
- `std::collections::HashMap` (for in-memory representation of state, e.g., `HashMap<Address, Account>`)
- `tracing` (for logging state transitions and errors)

This document lists the actionable development tasks for implementing RustChain's state machine logic.

## I. Module Definition and State Structures

- [ ] **Define `state_machine.rs` module.** (Corresponds to DEVELOPMENT_FLOW.md Phase 0)
- [ ] **Define `Account` struct.** (DEVELOPMENT_FLOW.md Phase 6)
    - Fields: `balance` (u64), `nonce` (u64).
    - Consider: `code_hash`, `storage_root` for future smart contract capabilities (not for minimal).
    - Implement `Serialize`, `Deserialize` (likely needed for storage).
- [ ] **Define `State` representation.** (DEVELOPMENT_FLOW.md Phase 6)
    - E.g., `type WorldState = HashMap<Address, Account>` for an in-memory representation.
    - This will be the primary structure the `StateMachine` operates on.
    - (The actual persistence of this state is handled by the Storage Layer).
- [ ] Define `StateMachine` struct to encapsulate state transition logic.
    - It will hold or have access to the current `WorldState`.
- [ ] Implement `new()` constructor for `StateMachine`, possibly taking an initial state (e.g. from genesis).

## II. Transaction Validation (Stateful)

- [ ] **Implement stateful transaction validation.** (DEVELOPMENT_FLOW.md Phase 2 & 6)
    - Function signature: `validate_transaction_stateful(transaction: &Transaction, current_state: &WorldState) -> Result<(), StateMachineError>`.
    - **Validate sender account existence and nonce:** (DEVELOPMENT_FLOW.md Phase 6)
        - Check if sender account exists.
        - Check if transaction nonce matches account nonce.
    - **Validate sender balance:** (DEVELOPMENT_FLOW.md Phase 6)
        - Check if sender has sufficient balance for the transaction amount (+ gas/fees in future).
    - Signature verification (already in `Transaction` struct, but can be re-verified here as a pre-condition if desired or if context demands).

## III. Transaction Application & State Updates

- [ ] **Implement transaction application logic.** (DEVELOPMENT_FLOW.md Phase 6)
    - Function signature: `apply_transaction(transaction: &Transaction, current_state: &mut WorldState) -> Result<(), StateMachineError>`.
    - This function assumes the transaction has already passed `validate_transaction_stateful`.
    - **Update sender account:** Deduct amount, increment nonce.
    - **Update/Create recipient account:** (DEVELOPMENT_FLOW.md Phase 6)
        - If recipient account does not exist, create it.
        - Add amount to recipient's balance.
- [ ] Implement `apply_block(block: &Block, current_state: &mut WorldState) -> Result<(), StateMachineError>`.
    - Iterates through transactions in the block.
    - For each transaction: `validate_transaction_stateful`, then `apply_transaction`.
    - Handle errors: decide on atomicity (e.g., if one tx fails, does the whole block fail to apply? For minimal, yes).

## IV. State Queries

- [ ] Implement functions to query account details from the state:
    - `get_account(address: &Address, current_state: &WorldState) -> Option<&Account>`.
    - `get_balance(address: &Address, current_state: &WorldState) -> u64` (returns 0 if account doesn't exist).
    - `get_nonce(address: &Address, current_state: &WorldState) -> u64` (returns 0 if account doesn't exist).

## V. Error Handling

- [ ] Define `StateMachineError` enum.
    - E.g., `AccountNotFound`, `InsufficientBalance`, `IncorrectNonce`, `InvalidSignature` (if re-checked), `InternalError`.
    - This might overlap or extend `TxValidationError` from `core_data_structures` for state-specific issues.

## VI. Unit Tests

- [ ] Test `Account` struct (de)serialization (if applicable).
- [ ] Test stateful transaction validation:
    - Valid transaction.
    - Invalid nonce (too high, too low).
    - Insufficient balance.
    - Sender account does not exist.
- [ ] Test transaction application:
    - Simple transfer between existing accounts.
    - Transfer creating a new recipient account.
- [ ] Test block application:
    - Block with multiple valid transactions.
    - Block with an invalid transaction (should fail to apply the block or specific tx, depending on design).
- [ ] Test state query functions.
- [ ] ✅ **Milestone Check:** Account balances and nonces are correctly updated based on processing transactions within a block. (Corresponds to DEVELOPMENT_FLOW.md Phase 6 Milestone: "Balances updated based on txs in block")

## VII. Logging

- [ ] Add `tracing` logs for key state machine operations: applying transaction, applying block, state changes (e.g., account created, balance updated). 