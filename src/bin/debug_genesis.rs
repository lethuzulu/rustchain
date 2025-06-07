use rustchain::wallet::address_from_public_key;
use rustchain::types::{PublicKey};
use ed25519_dalek::VerifyingKey;

fn main() -> anyhow::Result<()> {
    let genesis_validator_hex = "68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2";
    
    println!("Genesis validator hex: {}", genesis_validator_hex);
    
    // This is what the genesis parsing code does
    let public_key_bytes = hex::decode(genesis_validator_hex)
        .map_err(|e| anyhow::anyhow!("Invalid validator public key hex: {}", e))?;
    
    println!("Decoded bytes length: {}", public_key_bytes.len());
    println!("Decoded bytes: {:?}", public_key_bytes);
    
    if public_key_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Validator public key must be 32 bytes"));
    }
    
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
        .map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;
    
    let public_key = PublicKey(verifying_key);
    let derived_address = address_from_public_key(&public_key);
    
    println!("Derived address from genesis public key: {}", hex::encode(derived_address.0));
    
    Ok(())
} 