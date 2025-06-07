use rustchain::wallet::{Wallet, address_from_public_key};
use rustchain::types::{Address, PublicKey};

fn main() -> anyhow::Result<()> {
    let key_path = "dev/node1-validator.key";
    
    println!("Loading validator key from: {}", key_path);
    
    // Load the wallet from the key file
    let wallet = Wallet::load_from_file(key_path)?;
    
    // Get the public key and address
    let public_key = wallet.public_key();
    let address = wallet.address();
    
    println!("Public key: {}", hex::encode(public_key.0.as_bytes()));
    println!("Address: {}", hex::encode(address.0));
    
    // Also test the standalone address derivation function
    let derived_address = address_from_public_key(public_key);
    println!("Derived address: {}", hex::encode(derived_address.0));
    
    // Check if they match
    if address == &derived_address {
        println!("✅ Address derivation is consistent");
    } else {
        println!("❌ Address derivation mismatch!");
    }
    
    Ok(())
} 