use serde::{Serialize, Deserialize};
use crate::types::{Address, Signature, Nonce, Hash, PublicKey};
use bincode::{Encode, config};
use sha2::{Sha256, Digest};
use anyhow::Context; // For context on errors if needed
use thiserror::Error; // Using thiserror for convenience

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode)]
pub struct Transaction {
    pub sender: Address, // Should ideally be derived from the PublicKey that signed, or PublicKey itself
    pub recipient: Address,
    pub amount: u64,
    pub nonce: Nonce,
    pub signature: Signature,
}

/// Represents errors that can occur during transaction validation (stateless checks).
#[derive(Debug, Clone, PartialEq, Eq, Error)] // Using thiserror for convenience
pub enum TxValidationError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Transaction amount must be greater than zero")]
    ZeroAmount,
    #[error("Sender and recipient address cannot be the same")]
    SenderIsRecipient,
    // Add more stateless validation errors here if needed (e.g., amount is zero)
}

// Internal struct for canonical serialization for signing
#[derive(Serialize, Encode)] // Serde for bincode, bincode::Encode for bincode 2.x
struct TransactionSignablePayload<'a> {
    sender: &'a Address, // Or PublicKey if sender field in Tx becomes PublicKey
    recipient: &'a Address,
    amount: u64,
    nonce: &'a Nonce,
}

impl Transaction {
    /// Creates a new transaction.
    /// The signature is typically added after creation by the sender.
    /// The hash would also be computed after all fields are set.
    pub fn new(sender: Address, recipient: Address, amount: u64, nonce: Nonce, signature: Signature) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            nonce,
            signature,
        }
    }

    /// Calculates the hash of the transaction data that is meant to be signed.
    /// This typically excludes the signature itself.
    pub fn data_to_sign_hash(&self) -> anyhow::Result<Hash> {
        let payload = TransactionSignablePayload {
            sender: &self.sender,
            recipient: &self.recipient,
            amount: self.amount,
            nonce: &self.nonce,
        };
        let bincode_config = config::standard();
        let serialized_payload = bincode::encode_to_vec(&payload, bincode_config)
            .map_err(|e| TxValidationError::SerializationError(e.to_string()))
            .context("Failed to serialize transaction payload for signing hash")?;

        let mut hasher = Sha256::new();
        hasher.update(&serialized_payload);
        Ok(Hash(hasher.finalize().into()))
    }

    /// Verifies the transaction's signature against the sender's public key.
    /// Assumes the `sender` field in the transaction can be used to retrieve/identify the public key.
    /// For now, it takes PublicKey directly.
    pub fn verify_signature(&self, sender_public_key: &PublicKey) -> Result<(), TxValidationError> {
        let message_hash = self.data_to_sign_hash().map_err(|e| 
            TxValidationError::SerializationError(format!("Hashing for signature verification failed: {}", e))
        )?;
        
        sender_public_key.0.verify_strict(message_hash.as_ref(), &self.signature.0)
            .map_err(|_| TxValidationError::InvalidSignature)
    }

    /// Performs intrinsic property validation checks on the transaction.
    /// This does NOT verify the signature and does NOT check against world state.
    pub fn validate_intrinsic_properties(&self) -> Result<(), TxValidationError> {
        if self.amount == 0 {
            return Err(TxValidationError::ZeroAmount);
        }
        // Optional: Prohibit sending to oneself in simple transfers
        // if self.sender == self.recipient {
        //     return Err(TxValidationError::SenderIsRecipient);
        // }
        // Add other stateless checks if necessary (e.g., max amount, field formats if not covered by types)
        Ok(())
    }

    /// Performs comprehensive stateless validation: intrinsic properties and signature verification.
    /// Does NOT check against world state (use StateMachine for that).
    pub fn validate(&self, sender_public_key: &PublicKey) -> Result<(), TxValidationError> {
        self.validate_intrinsic_properties()?;
        self.verify_signature(sender_public_key)
    }

    /// Calculates the unique ID (hash) of the entire transaction, including the signature.
    pub fn id(&self) -> anyhow::Result<Hash> {
        let bincode_config = config::standard();
        let serialized_tx = bincode::encode_to_vec(self, bincode_config)
            .context("Failed to serialize full transaction for ID hashing")?;
        
        let mut hasher = Sha256::new();
        hasher.update(&serialized_tx);
        Ok(Hash(hasher.finalize().into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, PublicKey as TypesPublicKey, Signature as TypesSignature, Nonce as TypesNonce, Hash as TypesHash};
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    // Helper to create a wallet for testing
    struct TestWallet {
        signing_key: SigningKey,
        public_key: TypesPublicKey,
        address: Address,
    }

    impl TestWallet {
        fn new() -> Self {
            let mut csprng = OsRng;
            let signing_key = SigningKey::generate(&mut csprng);
            let verifying_key = signing_key.verifying_key();
            let public_key = TypesPublicKey(verifying_key);
            let address = Address(*verifying_key.as_bytes()); // Using PK bytes as address for simplicity
            TestWallet { signing_key, public_key, address }
        }

        fn sign_data_hash(&self, data_hash: &TypesHash) -> TypesSignature {
            let dalek_sig = self.signing_key.sign(data_hash.as_ref());
            TypesSignature(dalek_sig)
        }
    }

    #[test]
    fn transaction_hashing_and_signing_verification() -> anyhow::Result<()> {
        let sender_wallet = TestWallet::new();
        let recipient_address = TestWallet::new().address; // Dummy recipient
        let amount = 100u64;
        let nonce_val = TypesNonce(1);

        // Create the data to be signed
        let signable_payload = TransactionSignablePayload {
            sender: &sender_wallet.address,
            recipient: &recipient_address,
            amount,
            nonce: &nonce_val,
        };
        let bincode_config = config::standard();
        let serialized_payload = bincode::encode_to_vec(&signable_payload, bincode_config)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized_payload);
        let data_hash = TypesHash(hasher.finalize().into());

        // Sign the hash
        let signature = sender_wallet.sign_data_hash(&data_hash);

        // Create the transaction
        let tx = Transaction::new(
            sender_wallet.address,
            recipient_address,
            amount,
            nonce_val,
            signature
        );

        // 1. Verify data_to_sign_hash()
        assert_eq!(tx.data_to_sign_hash()?, data_hash, "data_to_sign_hash mismatch");

        // 2. Verify signature (direct call)
        assert!(tx.verify_signature(&sender_wallet.public_key).is_ok(), "Signature verification failed");

        // 3. Verify ID hash (should be different from data_to_sign_hash)
        let tx_id = tx.id()?;
        println!("Transaction ID: {}", tx_id);
        assert_ne!(tx_id, data_hash, "Transaction ID should be different from data_to_sign_hash");
        
        // Tamper with the transaction and check signature verification fails
        let mut tampered_tx = tx.clone();
        tampered_tx.amount = 200;
        assert!(tampered_tx.validate_intrinsic_properties().is_ok(), "Intrinsic validation should pass for tampered amount if not zero");
        assert!(tampered_tx.verify_signature(&sender_wallet.public_key).is_err(), "Signature verification should fail for tampered tx");
        assert_eq!(tampered_tx.validate(&sender_wallet.public_key), Err(TxValidationError::InvalidSignature), "Full validation should fail for tampered tx due to signature");

        Ok(())
    }

     #[test]
    fn transaction_id_is_consistent() -> anyhow::Result<()> {
        let sender_wallet = TestWallet::new();
        let recipient_address = TestWallet::new().address;
        let amount = 50u64;
        let nonce = TypesNonce(2);
        let signature = sender_wallet.sign_data_hash(&TypesHash([0u8; 32])); // Dummy signature for this test
        
        let tx1 = Transaction::new(sender_wallet.address, recipient_address, amount, nonce, signature);
        let tx1_again = Transaction::new(sender_wallet.address, recipient_address, amount, nonce, signature);

        assert_eq!(tx1.id()?, tx1_again.id()?, "Transaction ID should be consistent for identical transactions");

        let mut tx2 = tx1.clone();
        tx2.amount = 51; // Change amount
        assert_ne!(tx1.id()?, tx2.id()?, "Transaction ID should change if amount changes");
        
        Ok(())
    }

    #[test]
    fn stateless_transaction_validation() { // Renamed this test to reflect its focus
        let sender_wallet = TestWallet::new();
        let recipient_address = TestWallet::new().address;

        // Valid transaction (intrinsic properties perspective)
        let tx_valid_props = Transaction::new(
            sender_wallet.address,
            recipient_address,
            100,
            TypesNonce(1),
            sender_wallet.sign_data_hash(&TypesHash([0u8; 32])) // Dummy signature for intrinsic checks
        );
        assert!(tx_valid_props.validate_intrinsic_properties().is_ok());

        // Transaction with zero amount
        let tx_zero_amount = Transaction::new(
            sender_wallet.address,
            recipient_address,
            0, // Zero amount
            TypesNonce(1),
            sender_wallet.sign_data_hash(&TypesHash([0u8; 32]))
        );
        assert_eq!(tx_zero_amount.validate_intrinsic_properties(), Err(TxValidationError::ZeroAmount));
        
        // Test the comprehensive validate method
        let data_hash_for_valid_sig = tx_valid_props.data_to_sign_hash().unwrap();
        let valid_signature = sender_wallet.sign_data_hash(&data_hash_for_valid_sig);

        let tx_fully_valid = Transaction::new(
            sender_wallet.address,
            recipient_address,
            100,
            TypesNonce(1),
            valid_signature
        );
        assert!(tx_fully_valid.validate(&sender_wallet.public_key).is_ok(), "Full validation failed for valid tx");

        let tx_bad_sig = Transaction::new(
            sender_wallet.address,
            recipient_address,
            100,
            TypesNonce(1),
            sender_wallet.sign_data_hash(&TypesHash([1u8; 32])) // Signature for different data
        );
        assert_eq!(tx_bad_sig.validate(&sender_wallet.public_key), Err(TxValidationError::InvalidSignature), "Full validation should fail for bad signature");

        let tx_zero_amount_full_val = Transaction::new(
            sender_wallet.address,
            recipient_address,
            0, 
            TypesNonce(1),
            valid_signature // Signature might be valid for zero amount, but intrinsic check should fail first
        );
        // The validate() method calls validate_intrinsic_properties() first.
        assert_eq!(tx_zero_amount_full_val.validate(&sender_wallet.public_key), Err(TxValidationError::ZeroAmount), "Full validation should fail for zero amount before checking signature");
    }
}
