use crate::transaction::Transaction;
use crate::types::{Address, BlockHeight, Hash, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use bincode::{self, Encode, Decode};
use sha2::{Sha256, Digest};
use thiserror::Error; // For custom errors

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BlockValidationError {
    #[error("Merkle root mismatch: expected {expected}, got {actual}")]
    MerkleRootMismatch { expected: Hash, actual: Hash },
    #[error("Block hash calculation error: {0}")]
    HashCalculationError(String),
    #[error("Serialization error for hashing: {0}")]
    SerializationError(String),
    #[error("Merkle tree construction failed to produce a root hash")]
    MerkleRootConstructionFailed,
    #[error("Transaction ID calculation failed during Merkle root construction: {0}")]
    TransactionIdError(String),
}

/// Represents the header of a block in the blockchain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct BlockHeader {
    pub parent_hash: Hash,          // Hash of the previous block's header
    pub block_number: BlockHeight,
    pub timestamp: Timestamp,             // Unix timestamp (seconds since epoch)
    pub tx_root: Hash,              // Merkle root of transactions in the block body
    pub validator: Address,         // Public address of the block's proposer/validator
    // The signature is of the BlockHeaderSignablePayload (i.e., header excluding this signature field).
    pub signature: Signature,       
}

/// Internal struct for canonical serialization of BlockHeader for signing and hashing.
/// The header's own signature is excluded from this payload.
#[derive(Serialize, Encode)] // Serde for bincode, bincode::Encode for bincode 2.x
struct BlockHeaderSignablePayload<'a> {
    parent_hash: &'a Hash,
    block_number: BlockHeight, // Assuming BlockHeight is Copy
    timestamp: Timestamp,    // Assuming Timestamp is Copy
    tx_root: &'a Hash,
    validator: &'a Address,
}

impl BlockHeader {
    /// Calculates the hash of the block header data that is meant to be signed by the validator
    /// and also serves as the block's unique ID (often called block hash).
    /// This typically excludes the signature itself.
    pub fn calculate_hash(&self) -> Result<Hash, bincode::error::EncodeError> {
        let mut header_clone_for_hashing = self.clone();
        // The signature must be empty when hashing to get the hash that was signed.
        header_clone_for_hashing.signature = Signature(vec![]); 
        
        // Using bincode for serialization before hashing
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&header_clone_for_hashing, config)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        let result = hasher.finalize();
        Ok(Hash(result.into()))
    }
}

/// Represents a block in the blockchain, containing a header and a list of transactions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Creates a new block with the given transactions, parent hash, validator, etc.
    /// This will calculate the Merkle root for the transactions and populate the header.
    /// The validator_signature must be provided externally after the block (and its hash) is constructed.
    pub fn new(
        parent_hash: Hash,
        block_number: BlockHeight,
        timestamp: Timestamp,
        validator: Address,
        transactions: Vec<Transaction>,
        validator_signature: Signature, // Signature over the header's hash (excluding this field)
    ) -> Result<Self, BlockValidationError> {
        let tx_root = calculate_merkle_root(&transactions)?;
        let header = BlockHeader {
            parent_hash,
            block_number,
            timestamp,
            tx_root,
            validator,
            signature: validator_signature, // This signature is on the hash of the other header fields
        };
        // Note: The provided signature should have been created *after* knowing all other header fields,
        // typically by calling header.calculate_hash() on a header *without* the signature field,
        // signing that hash, and then instantiating the final header with that signature.
        // This 'new' function assumes the signature is correctly pre-calculated and provided.

        Ok(Block {
            header,
            transactions,
        })
    }

    /// Verifies the block's integrity by checking if the `tx_root` in the header
    /// matches the calculated Merkle root of its transactions.
    pub fn verify_merkle_root(&self) -> Result<(), BlockValidationError> {
        let calculated_root = calculate_merkle_root(&self.transactions)?;
        if self.header.tx_root == calculated_root {
            Ok(())
        } else {
            Err(BlockValidationError::MerkleRootMismatch {
                expected: self.header.tx_root,
                actual: calculated_root,
            })
        }
    }
}

/// Calculates the Merkle root for a list of transactions.
///
/// # Arguments
/// * `transactions` - A slice of transactions to include in the Merkle tree.
///
/// # Returns
/// * `Ok(Hash)` - The calculated Merkle root.
/// * `Err(anyhow::Error)` - If any transaction ID calculation fails.
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<Hash, BlockValidationError> {
    if transactions.is_empty() {
        // Conventionally, the Merkle root of an empty set of transactions is a hash of an empty string or a zero hash.
        // Let's use a hash of an empty byte array for consistency.
        let mut hasher = Sha256::new();
        hasher.update(&[] as &[u8]);
        return Ok(Hash(hasher.finalize().into()));
    }

    let mut current_level_hashes: Vec<Hash> = transactions
        .iter()
        .map(|tx| tx.id().map_err(|e| BlockValidationError::TransactionIdError(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;

    if current_level_hashes.len() == 1 { // Single transaction, its hash is duplicated for pairing
        let single_tx_hash = current_level_hashes[0];
        let mut hasher = Sha256::new();
        hasher.update(single_tx_hash.as_ref());
        hasher.update(single_tx_hash.as_ref());
        return Ok(Hash(hasher.finalize().into()));
    }
    
    // If odd number of hashes (and more than 1), duplicate the last one before pairing.
    if current_level_hashes.len() % 2 != 0 {
        if let Some(last_hash) = current_level_hashes.last().cloned() {
            current_level_hashes.push(last_hash);
        }
    }

    while current_level_hashes.len() > 1 {
        let mut next_level_hashes = Vec::new();
        for chunk in current_level_hashes.chunks(2) {
            let left = chunk[0];
            let right = chunk[1];

            let mut hasher = Sha256::new();
            hasher.update(left.as_ref());
            hasher.update(right.as_ref());
            next_level_hashes.push(Hash(hasher.finalize().into()));
        }
        current_level_hashes = next_level_hashes;

        if current_level_hashes.len() % 2 != 0 && current_level_hashes.len() > 1 {
            if let Some(last_hash) = current_level_hashes.last().cloned() {
                current_level_hashes.push(last_hash);
            }
        }
    }

    current_level_hashes.pop().ok_or(BlockValidationError::MerkleRootConstructionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use crate::types::{Address, BlockHeight, Hash, Nonce, PublicKey, Signature as TypesSignature, Timestamp};
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    fn block_test_dummy_transaction(amount: u64, nonce_val: u64, salt: u8) -> (Transaction, PublicKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let sender_pk = PublicKey(verifying_key);
        let mut addr_bytes = *verifying_key.as_bytes();
        addr_bytes[0] = addr_bytes[0].wrapping_add(salt);
        let sender_address = Address(addr_bytes);
        let recipient_address = Address([1u8; 32]);

        let tx_for_hash_calc = Transaction {
            sender: sender_pk,
            recipient: recipient_address,
            amount,
            nonce: Nonce(nonce_val),
            signature: TypesSignature(signing_key.sign(&[salt]).to_bytes().to_vec()),
        };
        let data_hash = tx_for_hash_calc.data_to_sign_hash().expect("Data hash failed in dummy tx for block test");
        let final_signature = TypesSignature(signing_key.sign(data_hash.as_ref()).to_bytes().to_vec());

        (
            Transaction {
                sender: sender_pk,
                recipient: recipient_address,
                amount,
                nonce: Nonce(nonce_val),
                signature: final_signature,
            },
            sender_pk,
        )
    }

    fn dummy_signature() -> TypesSignature {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        TypesSignature(signing_key.sign(b"dummy_block_header_data").to_bytes().to_vec())
    }

    #[test]
    fn test_calculate_merkle_root_empty() -> Result<(), BlockValidationError> {
        let transactions = Vec::new();
        let root = calculate_merkle_root(&transactions)?;
        let mut hasher = Sha256::new();
        hasher.update(&[] as &[u8]);
        let expected_root = Hash(hasher.finalize().into());
        assert_eq!(root, expected_root);
        Ok(())
    }

    #[test]
    fn test_calculate_merkle_root_single_transaction() -> Result<(), BlockValidationError> {
        let (tx1, _) = block_test_dummy_transaction(100, 1, 0);
        let tx1_id = tx1.id().unwrap();
        let transactions = vec![tx1];
        let root = calculate_merkle_root(&transactions)?;
        let mut hasher = Sha256::new();
        hasher.update(tx1_id.as_ref());
        hasher.update(tx1_id.as_ref());
        let expected_root = Hash(hasher.finalize().into());
        assert_eq!(root, expected_root);
        Ok(())
    }

    #[test]
    fn test_calculate_merkle_root_two_transactions() -> Result<(), BlockValidationError> {
        let (tx1, _) = block_test_dummy_transaction(100, 1, 0);
        let (tx2, _) = block_test_dummy_transaction(200, 2, 1);
        let tx1_id = tx1.id().unwrap();
        let tx2_id = tx2.id().unwrap();
        let transactions = vec![tx1, tx2];
        let root = calculate_merkle_root(&transactions)?;
        let mut hasher = Sha256::new();
        hasher.update(tx1_id.as_ref());
        hasher.update(tx2_id.as_ref());
        let expected_root = Hash(hasher.finalize().into());
        assert_eq!(root, expected_root);
        Ok(())
    }

    #[test]
    fn test_calculate_merkle_root_three_transactions() -> Result<(), BlockValidationError> {
        let (tx1, _) = block_test_dummy_transaction(100, 1, 0);
        let (tx2, _) = block_test_dummy_transaction(200, 2, 1);
        let (tx3, _) = block_test_dummy_transaction(300, 3, 2);
        let tx1_id = tx1.id().unwrap();
        let tx2_id = tx2.id().unwrap();
        let tx3_id = tx3.id().unwrap();
        let transactions = vec![tx1, tx2, tx3];
        let root = calculate_merkle_root(&transactions)?;
        let mut h1 = Sha256::new(); h1.update(tx1_id.as_ref()); h1.update(tx2_id.as_ref()); let h12 = Hash(h1.finalize().into());
        let mut h2 = Sha256::new(); h2.update(tx3_id.as_ref()); h2.update(tx3_id.as_ref()); let h33 = Hash(h2.finalize().into());
        let mut hr = Sha256::new(); hr.update(h12.as_ref()); hr.update(h33.as_ref()); let expected_root = Hash(hr.finalize().into());
        assert_eq!(root, expected_root);
        Ok(())
    }
    
    #[test]
    fn test_calculate_merkle_root_four_transactions() -> Result<(), BlockValidationError> {
        let (tx1, _) = block_test_dummy_transaction(100, 1, 0);
        let (tx2, _) = block_test_dummy_transaction(200, 2, 1);
        let (tx3, _) = block_test_dummy_transaction(300, 3, 2);
        let (tx4, _) = block_test_dummy_transaction(400, 4, 3);
        let tx1_id = tx1.id().unwrap();
        let tx2_id = tx2.id().unwrap();
        let tx3_id = tx3.id().unwrap();
        let tx4_id = tx4.id().unwrap();
        let transactions = vec![tx1, tx2, tx3, tx4];
        let root = calculate_merkle_root(&transactions)?;

        let mut h1 = Sha256::new(); h1.update(tx1_id.as_ref()); h1.update(tx2_id.as_ref()); let h12 = Hash(h1.finalize().into());
        let mut h2 = Sha256::new(); h2.update(tx3_id.as_ref()); h2.update(tx4_id.as_ref()); let h34 = Hash(h2.finalize().into());
        let mut hr = Sha256::new(); hr.update(h12.as_ref()); hr.update(h34.as_ref()); let expected_root = Hash(hr.finalize().into());
        assert_eq!(root, expected_root);
        Ok(())
    }

    #[test]
    fn test_block_header_hash_consistency() -> Result<(), BlockValidationError> {
        let header1 = BlockHeader {
            parent_hash: Hash([1u8; 32]),
            block_number: BlockHeight(1),
            timestamp: Timestamp(100),
            tx_root: Hash([2u8; 32]),
            validator: Address([3u8; 32]),
            signature: dummy_signature(),
        };
        let header2 = header1.clone();
        assert_eq!(header1.calculate_hash().unwrap(), header2.calculate_hash().unwrap());
        let mut header3 = header1.clone();
        header3.timestamp = Timestamp(101);
        assert_ne!(header1.calculate_hash().unwrap(), header3.calculate_hash().unwrap());
        Ok(())
    }

    #[test]
    fn test_new_block_and_verify_merkle_root() -> Result<(), BlockValidationError> {
        let parent_hash = Hash([0u8; 32]);
        let block_number = BlockHeight(1);
        let timestamp = Timestamp(1234567890);
        let validator_addr = Address([1u8; 32]);
        let (tx1, _) = block_test_dummy_transaction(50, 1, 0);
        let (tx2, _) = block_test_dummy_transaction(70, 2, 1);
        let transactions = vec![tx1.clone(), tx2.clone()];

        let prospective_tx_root = calculate_merkle_root(&transactions)?;
        let header_payload_for_signing = BlockHeader {
            parent_hash,
            block_number,
            timestamp,
            tx_root: prospective_tx_root,
            validator: validator_addr,
            signature: dummy_signature(),
        };
        let header_hash_to_sign = header_payload_for_signing.calculate_hash().unwrap();
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let validator_signature = TypesSignature(signing_key.sign(header_hash_to_sign.as_ref()).to_bytes().to_vec());

        let block = Block::new(
            parent_hash, block_number, timestamp, validator_addr, 
            transactions.clone(),
            validator_signature
        )?;

        assert_eq!(block.header.tx_root, prospective_tx_root);
        block.verify_merkle_root()?;

        let mut wrong_tx_root_block = block.clone();
        wrong_tx_root_block.header.tx_root = Hash([9u8; 32]);
        match wrong_tx_root_block.verify_merkle_root() {
            Err(BlockValidationError::MerkleRootMismatch { expected, actual }) => {
                assert_eq!(expected, wrong_tx_root_block.header.tx_root);
                assert_eq!(actual, prospective_tx_root);
            }
            _ => panic!("Expected MerkleRootMismatch error"),
        }
        Ok(())
    }
}
