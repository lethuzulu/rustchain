# Development Fixtures

This directory contains test fixtures and configuration files for RustChain development.

## Genesis Files

### `test_genesis.json`
A single-validator development genesis configuration with:
- 1 validator for simplified testing
- 5 pre-funded accounts with varying balances
- Suitable for local development and testing

### Usage Examples

#### Start node with custom genesis:
```bash
cargo run -- node --genesis-file dev/test_genesis.json
```

#### Start node with default genesis:
```bash
cargo run -- node
```

## Test Accounts

The test genesis includes these pre-funded accounts:

| Address (hex) | Balance | Purpose |
|---------------|---------|---------|
| `d75a980...` | 10,000,000 | Validator account |
| `1111111...` | 1,000,000 | Test account 1 |
| `2222222...` | 500,000 | Test account 2 |
| `3333333...` | 250,000 | Test account 3 |
| `4444444...` | 100,000 | Test account 4 |

## Creating Test Transactions

You can create test transactions between these accounts using the wallet CLI:

```bash
# Generate a new wallet
cargo run -- wallet generate

# Send tokens (you'll need to create transactions with proper nonces)
cargo run -- wallet send --to 2222222222222222222222222222222222222222222222222222222222222222 --amount 1000
```

## Notes

- The validator public key in the genesis file is a placeholder for development
- In production, use proper key generation and secure key management
- The timestamp corresponds to January 1, 2022 00:00:00 UTC 