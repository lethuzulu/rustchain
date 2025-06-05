use crate::transaction::{Transaction, TxValidationError};
use crate::types::Hash;
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use thiserror::Error;
use tracing::debug;

/// Configuration for the Mempool.
#[derive(Debug, Clone, Copy)]
pub struct MempoolConfig {
    pub max_transactions: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        MempoolConfig {
            max_transactions: 1000, // Default to 1000 transactions
        }
    }
}

/// Errors that can occur within the Mempool.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MempoolError {
    #[error("Transaction already exists in the mempool: {0}")]
    TransactionExists(Hash),
    #[error("Mempool is full. Cannot add more transactions.")]
    PoolFull,
    #[error("Transaction failed stateless validation: {0:?}")]
    StatelessValidationFailed(TxValidationError),
    #[error("Transaction amount is zero, not allowed in mempool")]
    ZeroAmountTransaction,
    #[error("Internal mempool error: {0}")]
    Internal(String),
}

/// Represents the status of the mempool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolStatus {
    pub pending_transactions_count: usize,
    pub capacity: usize,
}


/// The Mempool stores transactions that are waiting to be included in a block.
#[derive(Debug)]
pub struct Mempool {
    config: MempoolConfig,
    inner: RwLock<MempoolInner>,
}

#[derive(Debug, Default)]
struct MempoolInner {
    transactions: HashMap<Hash, Transaction>,
    pending_queue: VecDeque<Hash>, // Stores transaction hashes in order of arrival (FIFO)
}

impl Mempool {
    /// Creates a new Mempool instance.
    pub fn new(config: MempoolConfig) -> Self {
        Mempool {
            config,
            inner: RwLock::new(MempoolInner::default()),
        }
    }

    /// Adds a transaction to the mempool after performing basic validation.
    ///
    /// # Arguments
    /// * `transaction` - The transaction to add.
    ///
    /// # Returns
    /// * `Ok(Hash)` - The hash of the added transaction if successful.
    /// * `Err(MempoolError)` - If the transaction is invalid, a duplicate, or the mempool is full.
    pub fn add_transaction(&self, transaction: Transaction) -> Result<Hash, MempoolError> {
        let tx_id = transaction.id().map_err(|e| MempoolError::Internal(format!("Failed to calculate transaction ID: {}", e)))?;

        let mut inner = self.inner.write().expect("Failed to acquire write lock on mempool");

        if inner.pending_queue.len() >= self.config.max_transactions {
            debug!("Mempool full. Cannot add transaction: {}", tx_id);
            return Err(MempoolError::PoolFull);
        }

        if inner.transactions.contains_key(&tx_id) {
            debug!("Transaction {} already exists in mempool.", tx_id);
            return Err(MempoolError::TransactionExists(tx_id));
        }

        // Basic mempool-specific validation: prevent zero-amount transactions.
        // More comprehensive stateless validation (like signature) should ideally be done before calling this.
        if transaction.amount == 0 {
            debug!("Transaction {} has zero amount, rejecting.", tx_id);
            return Err(MempoolError::ZeroAmountTransaction);
        }

        // TODO: Consider further stateless validation if needed, e.g. transaction.validate_intrinsic_properties()
        // For now, we assume prior validation or that the state machine will do more thorough checks.

        inner.transactions.insert(tx_id, transaction);
        inner.pending_queue.push_back(tx_id);

        debug!("Added transaction {} to mempool. Pending: {}", tx_id, inner.pending_queue.len());
        Ok(tx_id)
    }

    /// Retrieves a list of pending transactions suitable for inclusion in a new block.
    /// Transactions are selected based on FIFO order.
    ///
    /// # Arguments
    /// * `max_txs` - Maximum number of transactions to return.
    /// * `max_total_size_bytes` - (Optional) Maximum total serialized size of transactions. (Not implemented yet)
    ///
    /// # Returns
    /// * `Vec<Transaction>` - A vector of transactions.
    pub fn get_pending_transactions(&self, max_txs: usize) -> Vec<Transaction> {
        let inner = self.inner.read().expect("Failed to acquire read lock on mempool");
        
        let mut selected_transactions = Vec::with_capacity(std::cmp::min(max_txs, inner.pending_queue.len()));

        for tx_hash in inner.pending_queue.iter().take(max_txs) {
            if let Some(transaction) = inner.transactions.get(tx_hash) {
                selected_transactions.push(transaction.clone()); // Clone to return owned transactions
            } else {
                // This case should ideally not happen if mempool state is consistent.
                // If it does, it implies a hash was in the queue but its transaction was removed from the map.
                // This might happen if remove_transactions is not perfectly atomic or if there's a bug.
                tracing::warn!("Transaction hash {} found in pending_queue but not in transactions map. Mempool might be inconsistent.", tx_hash);
            }
        }
        debug!("Retrieved {} transactions for block creation. Requested max: {}", selected_transactions.len(), max_txs);
        selected_transactions
    }

    /// Removes transactions from the mempool, typically after they have been included in a block.
    ///
    /// # Arguments
    /// * `transaction_hashes` - A slice of transaction hashes to remove.
    pub fn remove_transactions(&self, transaction_hashes_to_remove: &[Hash]) {
        if transaction_hashes_to_remove.is_empty() {
            return;
        }

        let mut inner = self.inner.write().expect("Failed to acquire write lock on mempool for removal");
        
        let mut removed_count_map = 0;
        for tx_hash in transaction_hashes_to_remove {
            if inner.transactions.remove(tx_hash).is_some() {
                removed_count_map += 1;
            }
        }

        // Efficiently remove from VecDeque while preserving order for remaining items.
        // Create a HashSet for quick lookups of hashes to remove.
        let hashes_to_remove_set: std::collections::HashSet<_> = transaction_hashes_to_remove.iter().cloned().collect();
        let initial_queue_len = inner.pending_queue.len();
        inner.pending_queue.retain(|hash_in_queue| !hashes_to_remove_set.contains(hash_in_queue));
        
        let removed_from_queue_count = initial_queue_len - inner.pending_queue.len();

        debug!(
            "Removed {} transactions from map, {} entries from queue. Hashes to remove: {:?}. Pending: {}",
            removed_count_map,
            removed_from_queue_count,
            transaction_hashes_to_remove,
            inner.pending_queue.len()
        );
    }

    /// Checks if a transaction with the given hash exists in the mempool.
    pub fn contains_transaction(&self, tx_hash: &Hash) -> bool {
        let inner = self.inner.read().expect("Failed to acquire read lock on mempool");
        inner.transactions.contains_key(tx_hash)
    }

    /// Returns the current status of the mempool.
    pub fn status(&self) -> MempoolStatus {
        let inner = self.inner.read().expect("Failed to acquire read lock on mempool");
        MempoolStatus {
            pending_transactions_count: inner.pending_queue.len(),
            capacity: self.config.max_transactions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, Nonce, Signature as TypesSignature, PublicKey};
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    // Helper to create a dummy transaction for testing
    fn dummy_test_transaction(amount: u64, nonce_val: u64) -> (Transaction, PublicKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let sender_pk = PublicKey(verifying_key);
        let sender_address = Address(*verifying_key.as_bytes()); // Simple address from PK bytes
        let recipient_address = Address([1u8; 32]); // Dummy recipient

        let tx_to_sign = Transaction {
            sender: sender_address,
            recipient: recipient_address,
            amount,
            nonce: Nonce(nonce_val),
            signature: TypesSignature(signing_key.sign(&[])), // Dummy signature, will be replaced
        };

        // Calculate data_to_sign_hash
        let data_hash = tx_to_sign.data_to_sign_hash().expect("Failed to hash tx for signing");
        let signature = TypesSignature(signing_key.sign(data_hash.as_ref()));

        (
            Transaction {
                sender: sender_address,
                recipient: recipient_address,
                amount,
                nonce: Nonce(nonce_val),
                signature,
            },
            sender_pk,
        )
    }

    #[test]
    fn test_mempool_new() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);
        assert_eq!(mempool.config.max_transactions, 1000);
        let inner = mempool.inner.read().expect("Lock failed");
        assert!(inner.transactions.is_empty());
        assert!(inner.pending_queue.is_empty());
    }

    #[test]
    fn test_mempool_status() {
        let config = MempoolConfig { max_transactions: 5 };
        let mempool = Mempool::new(config);
        let status = mempool.status();
        assert_eq!(status.pending_transactions_count, 0);
        assert_eq!(status.capacity, 5);
    }

    #[test]
    fn test_add_transaction_success() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx, _sender_pk) = dummy_test_transaction(100, 1);
        let tx_id = tx.id().unwrap();

        match mempool.add_transaction(tx.clone()) {
            Ok(id) => {
                assert_eq!(id, tx_id);
                let inner = mempool.inner.read().unwrap();
                assert_eq!(inner.pending_queue.len(), 1);
                assert!(inner.transactions.contains_key(&tx_id));
                assert_eq!(inner.pending_queue.front().unwrap(), &tx_id);
            }
            Err(e) => panic!("Failed to add transaction: {:?}", e),
        }
        assert!(mempool.contains_transaction(&tx_id));
    }

    #[test]
    fn test_add_transaction_duplicate() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx, _sender_pk) = dummy_test_transaction(100, 1);
        
        mempool.add_transaction(tx.clone()).expect("First add should succeed");
        match mempool.add_transaction(tx.clone()) {
            Err(MempoolError::TransactionExists(id)) => {
                assert_eq!(id, tx.id().unwrap());
            }
            _ => panic!("Expected TransactionExists error"),
        }
        let inner = mempool.inner.read().unwrap();
        assert_eq!(inner.pending_queue.len(), 1, "Mempool should still have only one transaction after duplicate attempt");
    }

    #[test]
    fn test_add_transaction_pool_full() {
        let config = MempoolConfig { max_transactions: 1 };
        let mempool = Mempool::new(config);
        let (tx1, _) = dummy_test_transaction(100, 1);
        let (tx2, _) = dummy_test_transaction(200, 2); // Different transaction

        mempool.add_transaction(tx1).expect("First transaction should be added");
        
        match mempool.add_transaction(tx2) {
            Err(MempoolError::PoolFull) => (),
            _ => panic!("Expected PoolFull error"),
        }
    }

    #[test]
    fn test_add_transaction_zero_amount() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx_zero_amount, _sender_pk) = dummy_test_transaction(0, 1);

        match mempool.add_transaction(tx_zero_amount) {
            Err(MempoolError::ZeroAmountTransaction) => (),
            Ok(id) => panic!("Should not have added zero amount transaction, got id: {}", id),
            Err(e) => panic!("Expected ZeroAmountTransaction error, got {:?}", e),
        }
    }

    #[test]
    fn test_get_pending_transactions_empty() {
        let mempool = Mempool::new(MempoolConfig::default());
        let txs = mempool.get_pending_transactions(10);
        assert!(txs.is_empty());
    }

    #[test]
    fn test_get_pending_transactions_less_than_max() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
        mempool.add_transaction(tx1).unwrap();

        let (tx2, _) = dummy_test_transaction(20, 2);
        let tx2_id = tx2.id().unwrap();
        mempool.add_transaction(tx2).unwrap();

        let selected_txs = mempool.get_pending_transactions(5);
        assert_eq!(selected_txs.len(), 2);
        assert_eq!(selected_txs[0].id().unwrap(), tx1_id);
        assert_eq!(selected_txs[1].id().unwrap(), tx2_id);
    }

    #[test]
    fn test_get_pending_transactions_more_than_max() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
         mempool.add_transaction(tx1).unwrap();

        let (tx2, _) = dummy_test_transaction(20, 2);
        let tx2_id = tx2.id().unwrap();
        mempool.add_transaction(tx2).unwrap();

        let (tx3, _) = dummy_test_transaction(30, 3);
        mempool.add_transaction(tx3).unwrap();

        let selected_txs = mempool.get_pending_transactions(2);
        assert_eq!(selected_txs.len(), 2);
        assert_eq!(selected_txs[0].id().unwrap(), tx1_id);
        assert_eq!(selected_txs[1].id().unwrap(), tx2_id); // tx2_id should be here, not tx3_id due to FIFO
    }

    #[test]
    fn test_get_pending_transactions_respects_max_txs() {
        let mempool = Mempool::new(MempoolConfig::default());
        let mut tx_ids = Vec::new();
        for i in 0..5 {
            let (tx, _) = dummy_test_transaction(10 + i as u64, 1 + i as u64);
            tx_ids.push(tx.id().unwrap());
            mempool.add_transaction(tx).unwrap();
        }
        let selected_txs = mempool.get_pending_transactions(3);
        assert_eq!(selected_txs.len(), 3);
        assert_eq!(selected_txs[0].id().unwrap(), tx_ids[0]);
        assert_eq!(selected_txs[1].id().unwrap(), tx_ids[1]);
        assert_eq!(selected_txs[2].id().unwrap(), tx_ids[2]);

        let selected_txs_zero = mempool.get_pending_transactions(0);
        assert!(selected_txs_zero.is_empty());
    }

    #[test]
    fn test_remove_transactions_single() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
        mempool.add_transaction(tx1).unwrap();

        let (tx2, _) = dummy_test_transaction(20, 2);
        let tx2_id = tx2.id().unwrap();
        mempool.add_transaction(tx2.clone()).unwrap();

        assert!(mempool.contains_transaction(&tx1_id));
        mempool.remove_transactions(&[tx1_id]);
        assert!(!mempool.contains_transaction(&tx1_id));
        assert!(mempool.contains_transaction(&tx2_id)); // tx2 should still be there
        
        let inner = mempool.inner.read().unwrap();
        assert_eq!(inner.pending_queue.len(), 1);
        assert_eq!(inner.transactions.len(), 1);
        assert_eq!(inner.pending_queue.front().unwrap(), &tx2_id);
    }

    #[test]
    fn test_remove_transactions_multiple() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
        mempool.add_transaction(tx1).unwrap();

        let (tx2, _) = dummy_test_transaction(20, 2);
        let tx2_id = tx2.id().unwrap();
        mempool.add_transaction(tx2).unwrap();

        let (tx3, _) = dummy_test_transaction(30, 3);
        let tx3_id = tx3.id().unwrap();
        mempool.add_transaction(tx3.clone()).unwrap();

        mempool.remove_transactions(&[tx1_id, tx2_id]);
        assert!(!mempool.contains_transaction(&tx1_id));
        assert!(!mempool.contains_transaction(&tx2_id));
        assert!(mempool.contains_transaction(&tx3_id));

        let inner = mempool.inner.read().unwrap();
        assert_eq!(inner.pending_queue.len(), 1);
        assert_eq!(inner.transactions.len(), 1);
        assert_eq!(inner.pending_queue.front().unwrap(), &tx3_id);
    }

    #[test]
    fn test_remove_transactions_non_existent() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
        mempool.add_transaction(tx1).unwrap();

        let non_existent_hash = Hash([99u8; 32]);
        mempool.remove_transactions(&[non_existent_hash]); // Should not panic

        assert!(mempool.contains_transaction(&tx1_id));
        let inner = mempool.inner.read().unwrap();
        assert_eq!(inner.pending_queue.len(), 1);
        assert_eq!(inner.transactions.len(), 1);
    }

    #[test]
    fn test_remove_all_transactions() {
        let mempool = Mempool::new(MempoolConfig::default());
        let (tx1, _) = dummy_test_transaction(10, 1);
        let tx1_id = tx1.id().unwrap();
        mempool.add_transaction(tx1).unwrap();

        let (tx2, _) = dummy_test_transaction(20, 2);
        let tx2_id = tx2.id().unwrap();
        mempool.add_transaction(tx2).unwrap();

        mempool.remove_transactions(&[tx1_id, tx2_id]);
        assert!(!mempool.contains_transaction(&tx1_id));
        assert!(!mempool.contains_transaction(&tx2_id));

        let inner = mempool.inner.read().unwrap();
        assert!(inner.pending_queue.is_empty());
        assert!(inner.transactions.is_empty());
    }

    #[test]
    fn test_remove_transactions_maintains_order() {
        let mempool = Mempool::new(MempoolConfig::default());
        let mut tx_ids = Vec::new();
        for i in 0..5 {
            let (tx, _) = dummy_test_transaction(10 + i as u64, 1 + i as u64);
            let tx_id = tx.id().unwrap();
            tx_ids.push(tx_id);
            mempool.add_transaction(tx).unwrap();
        }

        // Remove tx_ids[1] and tx_ids[3]
        mempool.remove_transactions(&[tx_ids[1], tx_ids[3]]);

        let expected_order = vec![tx_ids[0], tx_ids[2], tx_ids[4]];
        let pending_txs = mempool.get_pending_transactions(5);
        let pending_ids: Vec<Hash> = pending_txs.iter().map(|tx| tx.id().unwrap()).collect();
        
        assert_eq!(pending_ids, expected_order, "Order of pending queue incorrect after removal");
        assert_eq!(pending_txs.len(), 3);
        let inner = mempool.inner.read().unwrap();
        assert_eq!(inner.transactions.len(), 3);
    }

    // More tests for add_transaction, get_pending_transactions, remove_transactions, etc.,
    // will be added as these functions are implemented.
}
