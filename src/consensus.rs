use crate::block::{Block, BlockHeader};
use crate::types::{address_from_public_key, Address, BlockHeight, Hash, PublicKey};
use ed25519_dalek::Verifier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Invalid block proposer: expected {expected:?}, got {got:?}")]
    InvalidProposer {
        expected: Address,
        got: Address,
    },
    #[error("Block signature is invalid")]
    InvalidSignature,
    #[error("Proposer is not in the validator set")]
    ProposerNotInValidatorSet,
    #[error("Fork choice error: {0}")]
    ForkChoiceError(String),
    #[error("Genesis block is not defined")]
    GenesisBlockUndefined,
    #[error("Internal consensus error: {0}")]
    InternalError(String),
}

/// The consensus engine for the blockchain.
/// For now, it implements a simple static Proof-of-Stake logic.
pub struct ConsensusEngine {
    /// A static list of validators' public keys.
    validators: Vec<PublicKey>,
}

impl ConsensusEngine {
    /// Creates a new consensus engine with a given set of static validators.
    pub fn new(validators: Vec<PublicKey>) -> Self {
        Self { validators }
    }

    /// Determines the expected proposer for a given block height using a round-robin schedule.
    pub fn get_proposer(&self, height: BlockHeight) -> Result<&PublicKey, ConsensusError> {
        if self.validators.is_empty() {
            return Err(ConsensusError::ProposerNotInValidatorSet);
        }
        let proposer_index = (height.0 as usize) % self.validators.len();
        Ok(&self.validators[proposer_index])
    }

    /// Validates a block's proposer against the round-robin schedule.
    pub fn validate_proposer(
        &self,
        block_header: &BlockHeader,
    ) -> Result<(), ConsensusError> {
        let expected_proposer_pk = self.get_proposer(block_header.block_number)?;
        let expected_address = address_from_public_key(expected_proposer_pk);

        if block_header.validator != expected_address {
            return Err(ConsensusError::InvalidProposer {
                expected: expected_address,
                got: block_header.validator,
            });
        }
        Ok(())
    }

    /// A simple longest-chain fork choice rule.
    /// Returns the hash of the preferred block header.
    pub fn fork_choice<'a>(
        &self,
        current_head: &'a BlockHeader,
        new_head: &'a BlockHeader,
    ) -> &'a BlockHeader {
        if new_head.block_number > current_head.block_number {
            new_head
        } else if new_head.block_number == current_head.block_number {
            // Tie-breaking rule: choose the one with the smaller hash.
            if new_head.calculate_hash().unwrap() < current_head.calculate_hash().unwrap() {
                new_head
            } else {
                current_head
            }
        } else {
            current_head
        }
    }

    /// Validates the entire block according to consensus rules.
    pub fn validate_block(&self, block: &Block) -> Result<(), ConsensusError> {
        // 1. Validate the proposer
        self.validate_proposer(&block.header)?;

        // 2. Verify the block signature
        let proposer_pk = self
            .get_proposer_pk_for_address(&block.header.validator)
            .ok_or(ConsensusError::ProposerNotInValidatorSet)?;
        let header_hash = block
            .header
            .calculate_hash()
            .map_err(|e| ConsensusError::InternalError(e.to_string()))?;
        if proposer_pk
            .0
            .verify(&header_hash.0, &block.header.signature.0)
            .is_err()
        {
            return Err(ConsensusError::InvalidSignature);
        }

        Ok(())
    }

    /// Finds the public key for a given validator address.
    fn get_proposer_pk_for_address(&self, address: &Address) -> Option<&PublicKey> {
        self.validators.iter().find(|pk| {
            let pk_address = address_from_public_key(pk);
            pk_address == *address
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;
    use crate::transaction::Transaction;
    use crate::types::{Nonce, Signature as TypesSignature};
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn generate_test_keypair() -> (SigningKey, PublicKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        (signing_key, PublicKey(verifying_key))
    }

    #[test]
    fn test_get_proposer() {
        let (_, pk1) = generate_test_keypair();
        let (_, pk2) = generate_test_keypair();
        let validators = vec![pk1, pk2];
        let consensus_engine = ConsensusEngine::new(validators);

        assert_eq!(
            consensus_engine.get_proposer(BlockHeight(0)).unwrap(),
            &pk1
        );
        assert_eq!(
            consensus_engine.get_proposer(BlockHeight(1)).unwrap(),
            &pk2
        );
        assert_eq!(
            consensus_engine.get_proposer(BlockHeight(2)).unwrap(),
            &pk1
        );
    }

    #[test]
    fn test_validate_proposer() {
        let (sk1, pk1) = generate_test_keypair();
        let (_, pk2) = generate_test_keypair();
        let validators = vec![pk1, pk2.clone()];
        let consensus_engine = ConsensusEngine::new(validators);

        let mut block_header = BlockHeader {
            parent_hash: Hash([0; 32]),
            block_number: BlockHeight(0),
            timestamp: crate::types::Timestamp(0),
            tx_root: Hash([0; 32]),
            validator: address_from_public_key(&pk1),
            signature: TypesSignature(sk1.sign(&[])),
        };

        assert!(consensus_engine.validate_proposer(&block_header).is_ok());

        block_header.block_number = BlockHeight(1);
        block_header.validator = address_from_public_key(&pk2);
        assert!(consensus_engine.validate_proposer(&block_header).is_ok());

        block_header.validator = address_from_public_key(&pk1);
        assert!(consensus_engine.validate_proposer(&block_header).is_err());
    }

    #[test]
    fn test_fork_choice() {
        let (sk1, pk1) = generate_test_keypair();
        let consensus_engine = ConsensusEngine::new(vec![pk1]);
        let mut header1 = BlockHeader {
            parent_hash: Hash([0; 32]),
            block_number: BlockHeight(10),
            timestamp: crate::types::Timestamp(0),
            tx_root: Hash([0; 32]),
            validator: address_from_public_key(&pk1),
            signature: TypesSignature(sk1.sign(&[])),
        };

        let mut header2 = header1.clone();
        header2.block_number = BlockHeight(11);

        assert_eq!(
            consensus_engine.fork_choice(&header1, &header2),
            &header2
        );
        assert_eq!(
            consensus_engine.fork_choice(&header2, &header1),
            &header2
        );

        header2.block_number = BlockHeight(10);
        // change hash
        header2.timestamp = crate::types::Timestamp(1);

        if header1.calculate_hash().unwrap() < header2.calculate_hash().unwrap() {
            assert_eq!(
                consensus_engine.fork_choice(&header1, &header2),
                &header1
            );
        } else {
            assert_eq!(
                consensus_engine.fork_choice(&header1, &header2),
                &header2
            );
        }
    }

    #[test]
    fn test_validate_block() {
        let (sk1, pk1) = generate_test_keypair();
        let (sk2, pk2) = generate_test_keypair();
        let validators = vec![pk1, pk2];
        let consensus_engine = ConsensusEngine::new(validators);
        let validator_address = address_from_public_key(&pk1);

        let mut block_header = BlockHeader {
            parent_hash: Hash([0; 32]),
            block_number: BlockHeight(0),
            timestamp: crate::types::Timestamp(0),
            tx_root: Hash([0; 32]),
            validator: validator_address,
            signature: TypesSignature(sk1.sign(&[])), // dummy signature
        };

        let header_hash = block_header.calculate_hash().unwrap();
        block_header.signature = TypesSignature(sk1.sign(&header_hash.0));

        let block = Block {
            header: block_header.clone(),
            transactions: Vec::new(),
        };

        assert!(consensus_engine.validate_block(&block).is_ok());

        // invalid signature
        let (sk_bad, _) = generate_test_keypair();
        let mut bad_block = block.clone();
        bad_block.header.signature = TypesSignature(sk_bad.sign(&header_hash.0));
        assert!(consensus_engine.validate_block(&bad_block).is_err());

        // invalid proposer
        let mut bad_block = block.clone();
        bad_block.header.block_number = BlockHeight(1);
        assert!(consensus_engine.validate_block(&bad_block).is_err());
    }
}
