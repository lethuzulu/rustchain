use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfiguration {
    /// Network configuration
    pub network: NodeNetworkConfig,
    /// Storage configuration
    pub storage: NodeStorageConfig,
    /// Consensus configuration
    pub consensus: NodeConsensusConfig,
    /// Validator configuration (optional)
    pub validator: Option<NodeValidatorConfig>,
    /// Genesis file path
    pub genesis_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeNetworkConfig {
    /// Port to listen on for P2P connections
    pub listen_port: u16,
    /// Listen address for P2P connections
    pub listen_addr: String,
    /// List of bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Maximum number of peers
    pub max_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStorageConfig {
    /// Database directory path
    pub db_path: String,
    /// Whether to create database if it doesn't exist
    pub create_if_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConsensusConfig {
    /// Block production interval in seconds
    pub block_interval: u64,
    /// Maximum transactions per block
    pub max_txs_per_block: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeValidatorConfig {
    /// Path to validator private key file
    pub private_key_path: String,
    /// Whether this node should act as a validator
    pub enabled: bool,
}

fn main() -> anyhow::Result<()> {
    let config_path = "dev/node1-config.toml";
    
    println!("Reading TOML file: {}", config_path);
    let config_content = fs::read_to_string(config_path)?;
    
    println!("TOML content:");
    println!("{}", config_content);
    
    println!("\nParsing TOML...");
    let config: NodeConfiguration = toml::from_str(&config_content)?;
    
    println!("Parsed config: {:#?}", config);
    
    Ok(())
} 