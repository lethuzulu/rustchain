use serde::{Deserialize, Serialize};
use bincode::Encode;
use std::fmt;

/// Represents a 32-byte SHA-256 hash.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, bincode::Decode)]
pub struct Hash(pub [u8; 32]);

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Hash").field(&hex::encode(self.0)).finish()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}


// --- Cryptography (Keys & Signatures) ---
// We will use types from the ed25519_dalek crate and wrap them if needed,
// or type alias them for convenience within our project.

/// Represents an Ed25519 public key.
/// We're using a newtype wrapper around ed25519_dalek::VerifyingKey for type safety

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey(pub ed25519_dalek::VerifyingKey);

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PublicKey")
            .field(&hex::encode(self.0.as_bytes()))
            .finish()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_bytes()))
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes() // ed25519_dalek::VerifyingKey has as_bytes()
    }
}

// Manual implementation of Encode for PublicKey
impl Encode for PublicKey {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.as_bytes().encode(encoder)
    }
}

/// Represents an Ed25519 signature
/// New type wrapper for ed25519::Signature
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(pub ed25519_dalek::Signature);

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Signature")
         .field(&hex::encode(self.0.to_bytes()))
         .finish()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_bytes()))
    }
}

// Manual implementation of Encode for Signature
impl Encode for Signature {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.to_bytes().encode(encoder)
    }
}

// --- Address ---

/// Represents a blockchain address, derived from a public key.
/// For simplicity, let's make it a fixed-size array of bytes,
/// potentially the same as a hash or derived from a public key.
/// Let's assume it's 32 bytes for now, similar to a hash.
/// (Alternatively, it could be derived from a PublicKey and be ~32 bytes if it's a hash of the PK,
/// or if we use the PK bytes directly, it would be the size of the PK).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, bincode::Decode)]
pub struct Address(pub [u8; 32]); // Assuming 32 bytes for now, e.g., a hash of a public key.

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Address")
         .field(&hex::encode(self.0))
         .finish()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0)) // Common "0x" prefix for addresses
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for Address {
    fn from(bytes: [u8; 32]) -> Self {
        Address(bytes)
    }
}

// TODO: Add a function to derive Address from PublicKey
// e.g., pub fn address_from_public_key(pk: &PublicKey) -> Address { ... }
// This would involve hashing the public key bytes.
// For now, Address is a generic 32-byte array

// --- Blockchain Specific Numeric Types ---


/// Represents the height of a block in the blockchain (sequential number).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Encode, bincode::Decode)]
pub struct BlockHeight(pub u64);

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for BlockHeight {
    fn from(val: u64) -> Self {
        BlockHeight(val)
    }
}

impl Into<u64> for BlockHeight {
    fn into(self) -> u64 {
        self.0
    }
}

/// Represents a Unix timestamp (seconds since epoch).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Encode, bincode::Decode)]
pub struct Timestamp(pub u64);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Potentially format this in a human-readable date/time string in the future,
        // but for now, just the raw number.
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Timestamp {
    fn from(val: u64) -> Self {
        Timestamp(val)
    }
}

impl Into<u64> for Timestamp {
    fn into(self) -> u64 {
        self.0
    }
}

/// Represents an account nonce for transaction ordering and replay protection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Encode, bincode::Decode)]
pub struct Nonce(pub u64);

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Nonce {
    fn from(val: u64) -> Self {
        Nonce(val)
    }
}

impl Into<u64> for Nonce {
    fn into(self) -> u64 {
        self.0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer; // For signing
    use ed25519_dalek::SigningKey; // The private key type
    use rand::rngs::OsRng; // For key generation
    use sha2::{Sha256, Digest};


    #[test]
    fn hash_creation_and_display() {
        let data = b"hello world";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let hash_val = Hash(result.into());

        println!("Generated Hash: {:?}", hash_val); // Uses Debug
        println!("Generated Hash (Display): {}", hash_val); // Uses Display
        assert_eq!(format!("{}", hash_val), hex::encode(result));
    }

    #[test]
    fn keypair_signature_and_address_derivation_placeholder() {
        use ed25519_dalek::Verifier;
        
        // Generate a keypair
        let mut csprng = OsRng{};
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let public_key_val = PublicKey(signing_key.verifying_key());

        println!("Public Key: {:?}", public_key_val);
        println!("Public Key (Display): {}", public_key_val);

        // Sign a message
        let message: &[u8] = b"This is a test message.";
        let signature_val = Signature(signing_key.sign(message));

        println!("Signature: {:?}", signature_val);
        println!("Signature (Display): {}", signature_val);

        // Verify the signature
        assert!(public_key_val.0.verify(message, &signature_val.0).is_ok());

        // Placeholder for Address derivation
        // For now, let's just create a dummy address from the public key's bytes (first 32 bytes)
        let pk_bytes = public_key_val.0.as_bytes();
        let mut address_bytes = [0u8; 32]; 
        // This is a simplification; if pk_bytes is shorter than 32, this will panic.
        // Ed25519 public keys are 32 bytes, so this is fine.
        address_bytes.copy_from_slice(&pk_bytes[..32]); 
        let address_val = Address(address_bytes);
        
        println!("Derived Address (Placeholder): {:?}", address_val);
        println!("Derived Address (Display): {}", address_val);
        assert_eq!(format!("{}", address_val), format!("0x{}", hex::encode(address_bytes)));

        // Example of creating an Address from a known hash (e.g. if address is a hash)
        let data_for_addr = b"address_data";
        let mut hasher_addr = Sha256::new();
        hasher_addr.update(data_for_addr);
        let addr_hash_result = hasher_addr.finalize();
        let address_from_hash = Address(addr_hash_result.into());
        println!("Address from hash: {}", address_from_hash);

    }

    #[test]
    fn numeric_types_creation_and_conversion() {
        let height_val: u64 = 100;
        let block_height = BlockHeight::from(height_val);
        assert_eq!(block_height.0, height_val);
        assert_eq!(block_height, BlockHeight(100));
        println!("Block Height: {}", block_height);

        let timestamp_val: u64 = 1678886400; // Example timestamp
        let ts = Timestamp::from(timestamp_val);
        assert_eq!(ts.0, timestamp_val);
        println!("Timestamp: {}", ts);

        let nonce_val: u64 = 5;
        let account_nonce = Nonce::from(nonce_val);
        assert_eq!(account_nonce.0, nonce_val);
        println!("Nonce: {}", account_nonce);

        let converted_height: u64 = block_height.into();
        assert_eq!(converted_height, height_val);
    }

    #[test]
    fn numeric_types_default() {
        assert_eq!(BlockHeight::default(), BlockHeight(0));
        assert_eq!(Timestamp::default(), Timestamp(0));
        assert_eq!(Nonce::default(), Nonce(0));
    }

}