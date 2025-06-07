use crate::block::Block;
use crate::transaction::Transaction;
use crate::types::{Address, Nonce, address_from_public_key};
use std::collections::HashMap;
use thiserror::Error;
use bincode::{Encode, Decode};

/// Represents an account in the world state.
#[derive(Clone, Debug, PartialEq, Eq, Default, Encode, Decode)]
pub struct Account {
    pub balance: u64,
    pub nonce: Nonce,
}

/// The entire state of the blockchain world.
pub type WorldState = HashMap<Address, Account>;

/// Errors that can occur in the state machine.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateMachineError {
    #[error("Account not found: {0:?}")]
    AccountNotFound(Address),
    #[error("Insufficient balance: has {current}, needs {required}")]
    InsufficientBalance { current: u64, required: u64 },
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: Nonce, actual: Nonce },
    #[error("Transaction validation error: {0}")]
    TransactionValidation(String),
    #[error("Incorrect nonce: expected {expected}, got {actual}")]
    IncorrectNonce { expected: Nonce, actual: Nonce },
}

/// The state machine is responsible for processing transactions and blocks
/// and updating the world state.
pub struct StateMachine {
    pub world_state: WorldState,
}

impl StateMachine {
    /// Creates a new state machine with an empty world state.
    pub fn new() -> Self {
        StateMachine {
            world_state: HashMap::new(),
        }
    }

    /// Creates a new state machine from a given world state.
    pub fn from_world_state(world_state: WorldState) -> Self {
        StateMachine { world_state }
    }

    /// Applies a single transaction to the world state.
    pub fn apply_transaction(
        &mut self,
        tx: &Transaction,
    ) -> Result<(), StateMachineError> {
        self.validate_transaction_stateful(tx)?;

        let sender_address = address_from_public_key(&tx.sender);
        let recipient_address = tx.recipient;

        // Decrement sender balance and increment nonce
        let sender_account = self
            .world_state
            .get_mut(&sender_address)
            .ok_or(StateMachineError::AccountNotFound(sender_address))?;
        sender_account.balance -= tx.amount;
        sender_account.nonce.0 += 1;

        // Increment recipient balance
        let recipient_account = self
            .world_state
            .entry(recipient_address)
            .or_insert_with(Account::default);
        recipient_account.balance += tx.amount;

        Ok(())
    }

    /// Validates a transaction against the current world state.
    pub fn validate_transaction_stateful(
        &self,
        tx: &Transaction,
    ) -> Result<(), StateMachineError> {
        let sender_address = address_from_public_key(&tx.sender);
        let sender_account = self
            .world_state
            .get(&sender_address)
            .ok_or(StateMachineError::AccountNotFound(sender_address))?;

        if sender_account.balance < tx.amount {
            return Err(StateMachineError::InsufficientBalance {
                current: sender_account.balance,
                required: tx.amount,
            });
        }

        if sender_account.nonce != tx.nonce {
            return Err(StateMachineError::InvalidNonce {
                expected: sender_account.nonce,
                actual: tx.nonce,
            });
        }

        Ok(())
    }

    /// Applies a block of transactions to the world state.
    /// If any transaction fails, the state is not modified.
    pub fn apply_block(&mut self, block: &Block) -> Result<(), StateMachineError> {
        let original_state = self.world_state.clone();
        for tx in &block.transactions {
            if let Err(e) = self.apply_transaction(tx) {
                self.world_state = original_state; // Revert state on failure
                return Err(e);
            }
        }
        Ok(())
    }

    /// Set an account in the world state (for genesis initialization)
    pub fn set_account(&mut self, address: Address, account: Account) {
        self.world_state.insert(address, account);
    }

    /// Get an account from the world state
    pub fn get_account(&self, address: &Address) -> Option<&Account> {
        self.world_state.get(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, BlockHeight, Nonce, PublicKey, Signature};
    use crate::wallet::Wallet;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use crate::block::{Block, BlockHeader};
    use crate::types::{Hash, Timestamp};

    fn generate_test_wallet() -> (SigningKey, Address) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let address = Address(signing_key.verifying_key().to_bytes());
        (signing_key, address)
    }

    #[test]
    fn test_apply_valid_transaction() {
        let (sender_sk, sender_addr) = generate_test_wallet();
        let (_, recipient_addr) = generate_test_wallet();

        let mut world_state = WorldState::new();
        world_state.insert(
            sender_addr,
            Account {
                balance: 1000,
                nonce: Nonce(0),
            },
        );

        let mut state_machine = StateMachine::from_world_state(world_state);

        let tx = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr,
            amount: 100,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test").to_bytes().to_vec()),
        };

        assert!(state_machine.apply_transaction(&tx).is_ok());

        let sender_account = state_machine.world_state.get(&sender_addr).unwrap();
        assert_eq!(sender_account.balance, 900);
        assert_eq!(sender_account.nonce, Nonce(1));

        let recipient_account = state_machine.world_state.get(&recipient_addr).unwrap();
        assert_eq!(recipient_account.balance, 100);
    }

    #[test]
    fn test_insufficient_balance() {
        let (sender_sk, sender_addr) = generate_test_wallet();
        let (_, recipient_addr) = generate_test_wallet();

        let mut world_state = WorldState::new();
        world_state.insert(
            sender_addr,
            Account {
                balance: 50,
                nonce: Nonce(0),
            },
        );

        let mut state_machine = StateMachine::from_world_state(world_state);

        let tx = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr,
            amount: 100,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test").to_bytes().to_vec()),
        };

        assert_eq!(
            state_machine.apply_transaction(&tx).unwrap_err(),
            StateMachineError::InsufficientBalance {
                current: 50,
                required: 100
            }
        );
    }

    #[test]
    fn test_invalid_nonce() {
        let (sender_sk, sender_addr) = generate_test_wallet();
        let (_, recipient_addr) = generate_test_wallet();

        let mut world_state = WorldState::new();
        world_state.insert(
            sender_addr,
            Account {
                balance: 1000,
                nonce: Nonce(5),
            },
        );

        let mut state_machine = StateMachine::from_world_state(world_state);
        let tx = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr,
            amount: 100,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test").to_bytes().to_vec()),
        };

        assert_eq!(
            state_machine.apply_transaction(&tx).unwrap_err(),
            StateMachineError::InvalidNonce {
                expected: Nonce(5),
                actual: Nonce(0)
            }
        );
    }

    #[test]
    fn test_apply_block() {
        let (sender_sk, sender_addr) = generate_test_wallet();
        let (_, recipient_addr1) = generate_test_wallet();
        let (_, recipient_addr2) = generate_test_wallet();

        let mut world_state = WorldState::new();
        world_state.insert(
            sender_addr,
            Account {
                balance: 1000,
                nonce: Nonce(0),
            },
        );

        let mut state_machine = StateMachine::from_world_state(world_state);

        let tx1 = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr1,
            amount: 100,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test1").to_bytes().to_vec()),
        };
        let tx2 = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr2,
            amount: 200,
            nonce: Nonce(1),
            signature: Signature(sender_sk.sign(b"test2").to_bytes().to_vec()),
        };

        let block = Block {
            header: crate::block::BlockHeader {
                parent_hash: Default::default(),
                block_number: crate::types::BlockHeight(1),
                timestamp: crate::types::Timestamp(0),
                tx_root: Default::default(),
                validator: Default::default(),
                signature: Signature(sender_sk.sign(b"block").to_bytes().to_vec()),
            },
            transactions: vec![tx1, tx2],
        };

        assert!(state_machine.apply_block(&block).is_ok());

        let sender_account = state_machine.world_state.get(&sender_addr).unwrap();
        assert_eq!(sender_account.balance, 700);
        assert_eq!(sender_account.nonce, Nonce(2));
    }

    #[test]
    fn test_apply_block_with_invalid_tx_reverts_state() {
        let (sender_sk, sender_addr) = generate_test_wallet();
        let (_, recipient_addr) = generate_test_wallet();

        let mut world_state = WorldState::new();
        world_state.insert(
            sender_addr,
            Account {
                balance: 1000,
                nonce: Nonce(0),
            },
        );
        let initial_state = world_state.clone();

        let mut state_machine = StateMachine::from_world_state(world_state);

        let tx1 = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr,
            amount: 100,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test1").to_bytes().to_vec()),
        };
        // Invalid nonce
        let tx2_invalid = Transaction {
            sender: PublicKey(sender_sk.verifying_key()),
            recipient: recipient_addr,
            amount: 200,
            nonce: Nonce(0),
            signature: Signature(sender_sk.sign(b"test2").to_bytes().to_vec()),
        };

        let block = Block {
            header: crate::block::BlockHeader {
                parent_hash: Default::default(),
                block_number: crate::types::BlockHeight(1),
                timestamp: crate::types::Timestamp(0),
                tx_root: Default::default(),
                validator: Default::default(),
                signature: Signature(sender_sk.sign(b"block").to_bytes().to_vec()),
            },
            transactions: vec![tx1, tx2_invalid],
        };

        assert!(state_machine.apply_block(&block).is_err());
        assert_eq!(state_machine.world_state, initial_state);
    }

    #[test]
    fn test_apply_transaction() {
        let mut sm = StateMachine::new();
        let sender_wallet = Wallet::new();
        let recipient_address = Address([2u8; 32]);

        // Add sender to state with initial balance
        let sender_address = address_from_public_key(&sender_wallet.public_key());
        sm.world_state.insert(sender_address, Account { balance: 1000, nonce: Nonce(0) });

        let tx = sender_wallet.create_signed_transaction(recipient_address, 100, Nonce(0)).unwrap();
        
        let result = sm.apply_transaction(&tx);
        assert!(result.is_ok());

        // Test insufficient balance
        let tx2 = sender_wallet.create_signed_transaction(recipient_address, 2000, Nonce(1)).unwrap();
        let result2 = sm.apply_transaction(&tx2);
        assert!(matches!(result2, Err(StateMachineError::InsufficientBalance { .. })));
    }

    #[test]
    fn test_incorrect_nonce() {
        let mut sm = StateMachine::new();
        let sender_wallet = Wallet::new();
        let recipient_address = Address([2u8; 32]);

        let sender_address = address_from_public_key(&sender_wallet.public_key());
        sm.world_state.insert(sender_address, Account { balance: 1000, nonce: Nonce(5) });

        let tx = sender_wallet.create_signed_transaction(recipient_address, 100, Nonce(0)).unwrap();

        let result = sm.apply_transaction(&tx);
        assert!(matches!(result, Err(StateMachineError::IncorrectNonce { .. })));
    }

    #[test]
    fn test_new_state_machine() {
        let mut sm = StateMachine::new();
        let sender_wallet = Wallet::new();
        let recipient_address = Address([2u8; 32]);
        let tx = Transaction::new(
            sender_wallet.public_key().clone(),
            recipient_address,
            100,
            Nonce(1),
            Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
        );
        let block = Block {
            header: BlockHeader {
                parent_hash: Hash([0; 32]),
                block_number: BlockHeight(1),
                timestamp: Timestamp(0),
                tx_root: Hash([0; 32]),
                validator: address_from_public_key(&sender_wallet.public_key()),
                signature: Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
            },
            transactions: vec![tx],
        };
        // Expect error because sender account does not exist
        assert!(sm.apply_block(&block).is_err());
    }

    #[test]
    fn test_block_with_invalid_tx_fails() {
        let mut sm = StateMachine::new();
        let sender_wallet = Wallet::new();
        let recipient_address = Address([2u8; 32]);
        let tx = Transaction::new(
            sender_wallet.public_key().clone(),
            recipient_address,
            100,
            Nonce(1),
            Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
        );
        let block = Block {
            header: BlockHeader {
                parent_hash: Hash([0; 32]),
                block_number: 1.into(),
                timestamp: Timestamp(0),
                tx_root: Hash([0; 32]),
                validator: address_from_public_key(&sender_wallet.public_key()),
                signature: Signature(ed25519_dalek::Signature::from_bytes(&[0; 64]).to_bytes().to_vec()),
            },
            transactions: vec![tx],
        };
        // Expect error because sender account does not exist
        assert!(sm.apply_block(&block).is_err());
    }
}
