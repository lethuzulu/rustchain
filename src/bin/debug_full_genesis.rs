use rustchain::wallet::address_from_public_key;
use rustchain::types::{PublicKey};
use ed25519_dalek::VerifyingKey;
use serde_json;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct GenesisData {
    pub validators: Vec<String>,
    pub initial_balances: HashMap<String, u64>,
    pub timestamp: u64,
    pub message: String,
}

fn main() -> anyhow::Result<()> {
    let genesis_file = "dev/test_genesis.json";
    
    println!("Loading genesis from: {}", genesis_file);
    
    // Load genesis file
    let genesis_content = std::fs::read_to_string(genesis_file)?;
    let genesis_data: GenesisData = serde_json::from_str(&genesis_content)?;
    
    println!("Genesis validators count: {}", genesis_data.validators.len());
    
    for (i, validator_hex) in genesis_data.validators.iter().enumerate() {
        println!("Validator {}: {}", i, validator_hex);
        
        // Parse as the main code does
        let public_key_bytes = hex::decode(validator_hex)
            .map_err(|e| anyhow::anyhow!("Invalid validator public key hex: {}", e))?;
        
        println!("  Decoded bytes length: {}", public_key_bytes.len());
        if public_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Validator public key must be 32 bytes"));
        }
        
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
            .map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;
        
        let public_key = PublicKey(verifying_key);
        let derived_address = address_from_public_key(&public_key);
        
        println!("  Derived address: {}", hex::encode(derived_address.0));
    }
    
    println!("\nInitial balances:");
    for (addr, balance) in &genesis_data.initial_balances {
        println!("  {}: {}", addr, balance);
    }
    
    Ok(())
} 