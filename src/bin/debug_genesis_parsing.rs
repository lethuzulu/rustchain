use rustchain::wallet::address_from_public_key;
use rustchain::types::{PublicKey};
use ed25519_dalek::VerifyingKey;

fn main() -> anyhow::Result<()> {
    let validator_hex = "68e8dfa9999a7d1de46d9ddbae29ebdca13fba0f8011661976e62bb69c133fb2";
    
    println!("Input validator hex: {}", validator_hex);
    
    // This is exactly what the genesis parsing code does
    let public_key_bytes = hex::decode(validator_hex)?;
    println!("Decoded bytes length: {}", public_key_bytes.len());
    
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())?;
    let public_key = PublicKey(verifying_key);
    
    println!("PublicKey.0.to_bytes() hex: {}", hex::encode(public_key.0.to_bytes()));
    
    let derived_address = address_from_public_key(&public_key);
    println!("Derived address: {}", hex::encode(derived_address.0));
    
    Ok(())
} 