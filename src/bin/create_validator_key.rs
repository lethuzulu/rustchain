use ed25519_dalek::SigningKey;
use rustchain::wallet::address_from_public_key;
use rustchain::types::{Address, PublicKey};
use std::fs::File;
use std::io::Write;
use sha2::Digest;

fn main() -> anyhow::Result<()> {
    // Target address from our genesis
    let target_address_hex = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let target_address_bytes = hex::decode(target_address_hex)?;
    let target_address = Address(target_address_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid address length"))?);
    
    println!("Creating validator key for address: {}", hex::encode(target_address.0));
    
    // For testing, let's create a deterministic key from a known seed
    // This is NOT secure for production but fine for testing
    let seed = b"rustchain_test_validator_seed_123";
    let mut hasher = sha2::Sha256::new();
    sha2::Digest::update(&mut hasher, seed);
    let hash = sha2::Digest::finalize(hasher);
    
    let hash_array: [u8; 32] = hash.into();
    let signing_key = SigningKey::from_bytes(&hash_array);
    let verifying_key = signing_key.verifying_key();
    let public_key = PublicKey(verifying_key);
    let derived_address = address_from_public_key(&public_key);
    
    println!("Generated key with address: {}", hex::encode(derived_address.0));
    
    if derived_address == target_address {
        println!("✅ Perfect match!");
    } else {
        println!("❌ Address mismatch - using generated key anyway for testing");
        println!("Expected: {}", hex::encode(target_address.0));
        println!("Got:      {}", hex::encode(derived_address.0));
    }
    
    // Save the private key
    let key_path = "dev/node1-validator.key";
    let mut file = File::create(key_path)?;
    file.write_all(&signing_key.to_bytes())?;
    
    println!("Validator key saved to: {}", key_path);
    println!("Public key: {}", hex::encode(verifying_key.to_bytes()));
    println!("Address: {}", hex::encode(derived_address.0));
    
    Ok(())
} 