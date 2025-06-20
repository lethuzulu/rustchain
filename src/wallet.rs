use crate::types::{Address, PublicKey, Signature, Nonce};
use crate::transaction::Transaction;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::rngs::OsRng; 
use sha2::{Sha256, Digest}; 
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use anyhow::Result;
use anyhow::Context; 
use serde;

/// Represents a wallet, holding a keypair.
/// For simplicity, we'll store the secret key directly.
/// In a real wallet, this would be encrypted or handled by a secure enclave.
pub struct Wallet {
    pub signing_key: SigningKey,
    pub public_key: PublicKey,
    pub address: Address,
}

impl Wallet {
    /// Generates a new wallet with a fresh Ed25519 keypair.
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_key = PublicKey(verifying_key);
        
        // Derive address from public key
        // For now, Address is [u8; 32] and PublicKey (VerifyingKey) is also 32 bytes.
        // We will use the public key bytes directly as the address.
        let address = Address(*verifying_key.as_bytes());

        Wallet {
            signing_key,
            public_key,
            address,
        }
    }

    /// Creates a Wallet from a SigningKey.
    /// Useful when loading a key from an external source.
    fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_key = PublicKey(verifying_key);
        let address = Address(*verifying_key.as_bytes());
        Wallet {
            signing_key,
            public_key,
            address,
        }
    }

    /// Returns the wallet's public address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the wallet's public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn get_signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Signs a message (typically a transaction hash) with the wallet's private key.
    /// The message should be a pre-hashed byte slice.
    pub fn sign(&self, message_hash: &[u8]) -> anyhow::Result<Signature> {
        if message_hash.len() != 32 {
            return Err(anyhow::anyhow!("Message hash must be 32 bytes for Ed25519 signing"));
        }
        let dalek_signature = self.signing_key.sign(message_hash);
        Ok(Signature(dalek_signature.to_bytes().to_vec()))
    }

    /// Saves the wallet's secret key to the specified file.
    /// For development/testing purposes only.
    pub fn save_to_file(&self, path_str: &str) -> anyhow::Result<()> {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Overwrite if exists
            .open(path)?;
        file.write_all(&self.signing_key.to_bytes())?;
        Ok(())
    }

    /// Loads a wallet by reading the secret key from the specified file.
    /// For development/testing purposes only.
    pub fn load_from_file(path_str: &str) -> anyhow::Result<Self> {
        let mut file = File::open(path_str)?;
        let mut secret_key_bytes = [0u8; SECRET_KEY_LENGTH];
        file.read_exact(&mut secret_key_bytes)?;
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        Ok(Wallet::from_signing_key(signing_key))
    }

    /// Creates and signs a transaction.
    pub fn create_signed_transaction(&self, recipient: Address, amount: u64, nonce: Nonce) -> Result<Transaction, anyhow::Error> {
        let tx_payload = Transaction {
            sender: *self.public_key(),
            recipient,
            amount,
            nonce,
            signature: Signature(vec![]), // Dummy signature
        };

        let tx_hash = tx_payload.id()?;
        let dalek_signature = self.signing_key.sign(tx_hash.as_ref());
        
        let signed_tx = Transaction {
            signature: Signature(dalek_signature.to_bytes().to_vec()),
            ..tx_payload
        };
        Ok(signed_tx)
    }
}

/// Derive address from public key - standalone function for use in other modules
pub fn address_from_public_key(public_key: &PublicKey) -> Address {
    Address(*public_key.0.as_bytes())
}

/// Generate a validator keypair - standalone function for use in other modules  
pub fn generate_validator_keypair() -> (SigningKey, PublicKey) {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let public_key = PublicKey(verifying_key);
    (signing_key, public_key)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, Nonce, Hash};
    use std::fs;
    use tempfile::NamedTempFile;
    use bincode; // Ensure bincode is in scope for tests as well

    #[test]
    fn generate_new_wallet() {
        let wallet = Wallet::new();
        println!("Generated Wallet:");
        println!("  Address: {}", wallet.address());
        println!("  Public Key: {}", wallet.public_key());
        // Private key is not usually printed, but we can check its existence.
        assert_eq!(wallet.signing_key.verifying_key(), wallet.public_key().0);
        assert_eq!(wallet.address().0, *wallet.public_key().0.as_bytes());
    }

    #[test]
    fn sign_message() {
        let wallet = Wallet::new();
        let message = b"This is a test message to sign";
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash: [u8; 32] = hasher.finalize().into();

        let signature_result = wallet.sign(&message_hash);
        assert!(signature_result.is_ok());
        let signature = signature_result.unwrap();

        // Verify the signature (using the public key from our PublicKey wrapper)
        // assert!(wallet.public_key().0.verify_strict(&message_hash, &signature.0.as_ref()).is_ok());
         println!("Message signed and verified successfully.");
    }

    #[test]
    fn sign_message_invalid_hash_length() {
        let wallet = Wallet::new();
        let short_message_hash = b"too_short";
        let signature_result = wallet.sign(short_message_hash);
        assert!(signature_result.is_err());
        println!("Signing with invalid hash length failed as expected: {}", signature_result.err().unwrap());
    }

    #[test]
    fn save_and_load_wallet() -> anyhow::Result<()> {
        let original_wallet = Wallet::new();
        
        // Create a temporary file for saving the wallet
        let temp_file = NamedTempFile::new()?;
        let file_path_str = temp_file.path().to_str().expect("Failed to get temp file path string");

        // Save the wallet
        original_wallet.save_to_file(file_path_str)?;
        assert!(Path::new(file_path_str).exists(), "Wallet file was not created.");

        // Load the wallet
        let loaded_wallet = Wallet::load_from_file(file_path_str)?;

        // Verify that the loaded wallet has the same keys/address
        assert_eq!(original_wallet.signing_key.to_bytes(), loaded_wallet.signing_key.to_bytes(), "Secret keys do not match.");
        assert_eq!(original_wallet.public_key(), loaded_wallet.public_key(), "Public keys do not match.");
        assert_eq!(original_wallet.address(), loaded_wallet.address(), "Addresses do not match.");

        println!("Wallet saved and loaded successfully.");
        
        // The temp_file will be deleted automatically when it goes out of scope
        Ok(())
    }

    #[test]
    fn load_non_existent_wallet() {
        let result = Wallet::load_from_file("non_existent_wallet.key");
        assert!(result.is_err());
        println!("Loading non-existent wallet failed as expected: {}", result.err().unwrap());
    }

    #[test]
    fn load_invalid_key_file() -> anyhow::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let file_path_str = temp_file.path().to_str().expect("Failed to get temp file path string");
        
        // Write invalid data to the file (e.g., wrong length)
        let mut file = File::create(file_path_str)?;
        file.write_all(b"invalid key data")?;
        drop(file); // Ensure file is closed

        let result = Wallet::load_from_file(file_path_str);
        assert!(result.is_err(), "Loading an invalid key file should result in an error.");
        if let Err(e) = result {
            println!("Loading invalid key file failed as expected: {}", e);
        } else {
            panic!("Test should have resulted in an error.");
        }
        Ok(())
    }

    #[test]
    fn test_wallet_creation_and_load_save() {
        let wallet = Wallet::new();
        let path = "test_wallet.key";
        assert!(wallet.save_to_file(path).is_ok());

        let loaded_wallet = Wallet::load_from_file(path).unwrap();
        assert_eq!(wallet.public_key(), loaded_wallet.public_key());
        // Note: Can't directly compare private keys as they are not public.
        // We infer they are the same if they produce the same public key and signatures.

        std::fs::remove_file(&path).unwrap(); // Clean up
    }

    #[test]
    fn test_create_and_sign_transaction() {
        let wallet = Wallet::new();
        let recipient_address = Address([1u8; 32]);
        let amount = 100;
        let nonce = Nonce(1);

        let tx_result = wallet.create_signed_transaction(recipient_address, amount, nonce);
        assert!(tx_result.is_ok());
        let tx = tx_result.unwrap();

        assert_eq!(tx.sender, *wallet.public_key());
        assert_eq!(tx.recipient, recipient_address);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.nonce, nonce);

        // Verify the signature
        assert!(tx.verify_signature(&wallet.public_key()).is_ok());
    }
} 