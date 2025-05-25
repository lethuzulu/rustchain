# 🔒 Consensus Protocol

RustChain uses a static Proof-of-Stake validator set with round-robin proposer rotation.

## 🎛️ Validator Selection

- At genesis: validator addresses defined in config.
- Each block: proposer is chosen via `(block_height % num_validators)`.

**Note:** For this minimal implementation, the validator set is static and defined at genesis. There are no on-chain mechanisms for staking, un-staking, rewards, or slashing. All validators in the set are assumed to have equal weight.

## 🧱 Block Proposal

- Selected validator:
    - Gathers txs from mempool
    - Computes Merkle root
    - Builds and signs block header
    - Broadcasts via `BlockMessage`

## ✅ Block Validation

- Signature must match expected proposer
- Header must include correct Merkle root
- All txs must be valid (correct nonce, sig, balance)

## ⛓️ Fork Choice

- Longest chain (highest block height)
- Tie-breaker: lowest block hash

## ⏱️ Proposer Timeout & Missed Slots

- For the initial minimal implementation, it is assumed that registered validators are online and will propose blocks in their designated slots.
- Robust handling of proposer timeouts, missed slots, and dynamic validator set changes are considered future enhancements beyond the minimal scope