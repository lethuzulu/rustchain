use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rustchain::wallet::Wallet;
use rustchain::types::{Address, Nonce}; // For parsing arguments

// Networking related imports
use rustchain::networking::{NetworkService, NetworkConfig, NetworkMessage, Libp2pPeerId};
use libp2p::identity;
use tokio::sync::mpsc;
use tracing_subscriber::fmt::format::FmtSpan;

// This brings the cli module into scope, which exports wallet_cli.
// wallet_cli, in turn, exports its own Cli and Commands structs.
mod cli;

use rustchain::consensus::ConsensusEngine;
use rustchain::state_machine::StateMachine;
use rustchain::storage::Storage;
use rustchain::mempool::{Mempool, MempoolConfig};
use rustchain::block::{Block, BlockHeader, calculate_merkle_root};
use rustchain::types::{BlockHeight, Hash, Signature, Timestamp, PublicKey};
use rustchain::wallet::{address_from_public_key, generate_validator_keypair};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::{Signer, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;

/// Genesis configuration data loaded from JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisData {
    /// List of initial validator public keys
    pub validators: Vec<String>, // Hex-encoded public keys
    /// Initial account balances 
    pub initial_balances: std::collections::HashMap<String, u64>, // Address -> Balance
    /// Genesis timestamp (Unix timestamp)
    pub timestamp: u64,
    /// Genesis block message
    pub message: String,
}

impl Default for GenesisData {
    fn default() -> Self {
        // Generate a default validator for development
        let (_, validator_public_key) = generate_validator_keypair();
        let validator_address = address_from_public_key(&validator_public_key);
        
        let mut initial_balances = std::collections::HashMap::new();
        initial_balances.insert(hex::encode(validator_address.0), 1000000); // 1M tokens for validator
        
        Self {
            validators: vec![hex::encode(validator_public_key.0.to_bytes())],
            initial_balances,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: "RustChain Genesis Block".to_string(),
        }
    }
}

/// Node configuration structure
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

impl Default for NodeConfiguration {
    fn default() -> Self {
        Self {
            network: NodeNetworkConfig {
                listen_port: 9000,
                listen_addr: "127.0.0.1".to_string(),
                bootstrap_peers: Vec::new(),
                max_peers: 50,
            },
            storage: NodeStorageConfig {
                db_path: "rustchain_db".to_string(),
                create_if_missing: true,
            },
            consensus: NodeConsensusConfig {
                block_interval: 5,
                max_txs_per_block: 10,
            },
            validator: None,
            genesis_file: None,
        }
    }
}

impl NodeConfiguration {
    /// Load configuration from TOML file, with CLI args taking precedence
    pub fn load_from_file_and_args(
        config_path: Option<&str>,
        node_args: &NodeArgs,
    ) -> anyhow::Result<Self> {
        // Start with defaults
        let mut config = if let Some(path) = config_path {
            tracing::info!("Loading configuration from: {}", path);
            let config_content = fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;
            let parsed_config = toml::from_str::<NodeConfiguration>(&config_content)
                .map_err(|e| anyhow::anyhow!("Failed to parse config TOML: {}", e))?;
            tracing::info!("Loaded config genesis_file: {:?}", parsed_config.genesis_file);
            parsed_config
        } else {
            NodeConfiguration::default()
        };

        // Override with CLI arguments
        if let Some(ref genesis_file) = node_args.genesis_file {
            config.genesis_file = Some(genesis_file.to_string_lossy().to_string());
        }
        
        if node_args.block_interval != 5 { // 5 is our default
            config.consensus.block_interval = node_args.block_interval;
        }
        
        if node_args.max_txs_per_block != 10 { // 10 is our default
            config.consensus.max_txs_per_block = node_args.max_txs_per_block;
        }

        if let Some(ref db_path) = node_args.db_path {
            config.storage.db_path = db_path.to_string_lossy().to_string();
        }

        if let Some(port) = node_args.port {
            config.network.listen_port = port;
        }

        if let Some(ref addr) = node_args.listen_addr {
            config.network.listen_addr = addr.clone();
        }

        if !node_args.bootstrap_peers.is_empty() {
            config.network.bootstrap_peers = node_args.bootstrap_peers.clone();
        }

        // Set up validator configuration
        if node_args.validator || node_args.validator_key.is_some() {
            let validator_config = NodeValidatorConfig {
                enabled: node_args.validator,
                private_key_path: node_args.validator_key
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "validator.key".to_string()),
            };
            config.validator = Some(validator_config);
        }

        Ok(config)
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage wallets (generate, show, send)
    #[clap(name = "wallet")] // Ensure the command is still 'wallet'
    WalletCmd(cli::wallet_cli::WalletCliArgs),
    /// Run a RustChain node
    Node(NodeArgs),
}

#[derive(Parser, Debug)]
struct NodeArgs {
    /// Path to configuration file (TOML format)
    #[clap(long, short = 'c')]
    pub config: Option<PathBuf>,

    /// Block production interval in seconds (default: 5)
    #[clap(long, default_value = "5")]
    pub block_interval: u64,
    
    /// Maximum transactions per block (default: 10)
    #[clap(long, default_value = "10")]
    pub max_txs_per_block: usize,

    /// Path to genesis configuration file
    #[clap(long)]
    pub genesis_file: Option<PathBuf>,

    /// Database directory path
    #[clap(long)]
    pub db_path: Option<PathBuf>,

    /// Network listen port
    #[clap(long)]
    pub port: Option<u16>,

    /// Network listen address
    #[clap(long)]
    pub listen_addr: Option<String>,

    /// Bootstrap peer addresses (can be specified multiple times)
    #[clap(long)]
    pub bootstrap_peers: Vec<String>,

    /// Validator private key file path
    #[clap(long)]
    pub validator_key: Option<PathBuf>,

    /// Enable validator mode
    #[clap(long)]
    pub validator: bool,
}

// Helper function to parse Address from hex string
fn parse_address(s: &str) -> Result<Address, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 { // 32 bytes = 64 hex chars
        return Err(format!("Address hex string must be 64 characters long, got {}", s.len()));
    }
    let mut bytes = [0u8; 32];
    hex::decode_to_slice(s, &mut bytes)
        .map_err(|e| format!("Invalid hex string for address: {}", e))?;
    Ok(Address(bytes))
}

/// Initialize genesis state from genesis data
async fn initialize_genesis_state(
    genesis_data: &GenesisData,
    storage: &Arc<Mutex<Storage>>,
    state_machine: &Arc<Mutex<StateMachine>>,
) -> anyhow::Result<()> {
    tracing::info!("Initializing genesis state...");

    // Parse and set initial account balances
    let mut state_machine_lock = state_machine.lock().await;
    for (address_hex, balance) in &genesis_data.initial_balances {
        let address = parse_address(address_hex)
            .map_err(|e| anyhow::anyhow!("Invalid address in genesis: {}", e))?;
        
        let account = rustchain::state_machine::Account {
            balance: *balance,
            nonce: Nonce(0),
        };
        
        state_machine_lock.set_account(address, account);
        tracing::info!("Genesis account: {} -> balance: {}", address_hex, balance);
    }
    drop(state_machine_lock);

    // Create genesis block
    let genesis_block = create_genesis_block(genesis_data)?;
    tracing::info!("Created genesis block with hash: {}", genesis_block.header.calculate_hash()?);

    // Store genesis block and state
    let storage_lock = storage.lock().await;
    storage_lock.put_block(&genesis_block)
        .map_err(|e| anyhow::anyhow!("Failed to store genesis block: {}", e))?;
    
    storage_lock.put_header_by_height(genesis_block.header.block_number.0, &genesis_block.header)
        .map_err(|e| anyhow::anyhow!("Failed to store genesis header: {}", e))?;
    
    storage_lock.set_chain_tip(&genesis_block.header.calculate_hash()?, genesis_block.header.block_number.0)
        .map_err(|e| anyhow::anyhow!("Failed to set genesis chain tip: {}", e))?;

    // Store initial account states
    let state_machine_lock = state_machine.lock().await;
    for (address_hex, balance) in &genesis_data.initial_balances {
        let address = parse_address(address_hex)
            .map_err(|e| anyhow::anyhow!("Invalid address in genesis initial balances: {}", e))?;
        let account = rustchain::state_machine::Account {
            balance: *balance,
            nonce: Nonce(0),
        };
        storage_lock.put_account(&address, &account)
            .map_err(|e| anyhow::anyhow!("Failed to store genesis account: {}", e))?;
    }
    drop(state_machine_lock);
    drop(storage_lock);

    tracing::info!("Genesis state initialized successfully!");
    Ok(())
}

/// Create the genesis block from genesis data
fn create_genesis_block(genesis_data: &GenesisData) -> anyhow::Result<Block> {
    // Genesis block has no transactions and no parent
    let transactions = Vec::new();
    let merkle_root = calculate_merkle_root(&transactions);
    
    // Parse the first validator as the genesis proposer
    let proposer_bytes = hex::decode(&genesis_data.validators[0])
        .map_err(|e| anyhow::anyhow!("Invalid proposer key in genesis: {}", e))?;
    if proposer_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Genesis proposer key must be 32 bytes"));
    }
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&proposer_bytes.try_into().unwrap())
        .map_err(|e| anyhow::anyhow!("Invalid Ed25519 proposer key: {}", e))?;
    let proposer = PublicKey(verifying_key);
    
    let header = BlockHeader {
        parent_hash: Hash([0u8; 32]), // Genesis has no parent
        block_number: BlockHeight(0),
        timestamp: Timestamp(genesis_data.timestamp),
        tx_root: merkle_root?,
        validator: address_from_public_key(&proposer),
        signature: Signature(vec![0u8; 64]), // Genesis block can have empty signature
    };

    Ok(Block {
        header,
        transactions,
    })
}

// Main entry point needs to be async if we call async functions directly within it.
// Or, we can keep main sync and use a tokio runtime builder if needed for more control.
// For simplicity, if run_node is the only async part for now, we can make main async.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging
    // Example: Basic subscriber, customize as needed (e.g., with RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_span_events(FmtSpan::CLOSE) // Log when spans close
        .init();

    let top_level_cli = Cli::parse();

    match top_level_cli.command {
        Commands::WalletCmd(wallet_cli_args) => {
            // This part remains synchronous as wallet operations are mostly file I/O
            cli::wallet_cli::run_wallet_cli(wallet_cli_args)?;
        }
        Commands::Node(node_args) => {
            // Load configuration from file and CLI args
            let config_path = node_args.config.as_ref().map(|p| p.to_string_lossy());
            let config_path_str = config_path.as_ref().map(|s| s.as_ref());
            let config = NodeConfiguration::load_from_file_and_args(config_path_str, &node_args)?;
            
            run_node(config).await?;
        }
    }

    Ok(())
}

async fn run_node(config: NodeConfiguration) -> anyhow::Result<()> {
    tracing::info!("Starting RustChain node with configuration: {:?}", config);

    // 1. Load Genesis Configuration
    let genesis_data = if let Some(ref genesis_path) = config.genesis_file {
        tracing::info!("Loading genesis from file: {}", genesis_path);
        let genesis_json = fs::read_to_string(genesis_path)
            .map_err(|e| anyhow::anyhow!("Failed to read genesis file: {}", e))?;
        serde_json::from_str::<GenesisData>(&genesis_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse genesis JSON: {}", e))?
    } else {
        tracing::info!("No genesis file specified, using default genesis configuration");
        GenesisData::default()
    };
    tracing::info!("Genesis loaded with {} validators and {} initial accounts", 
        genesis_data.validators.len(), 
        genesis_data.initial_balances.len()
    );

    // 2. Initialize Storage
    let storage = Arc::new(Mutex::new(
        Storage::new(&config.storage.db_path).map_err(|e| anyhow::anyhow!("Failed to initialize storage: {}", e))?,
    ));
    tracing::info!("Storage initialized at: {}", config.storage.db_path);

    // 3. Check if genesis needs to be initialized
    let needs_genesis = {
        let storage_lock = storage.lock().await;
        match storage_lock.get_chain_tip() {
            Ok(None) => {
                tracing::info!("No chain tip found, initializing genesis block");
                true
            }
            Ok(Some((_, height))) => {
                tracing::info!("Existing chain found with height: {}", height);
                false
            }
            Err(e) => {
                tracing::warn!("Error checking chain tip: {}. Assuming fresh database.", e);
                true
            }
        }
    };

    // 4. Initialize StateMachine with genesis state
    let state_machine = Arc::new(Mutex::new(StateMachine::new()));
    if needs_genesis {
        initialize_genesis_state(&genesis_data, &storage, &state_machine).await?;
    }
    tracing::info!("StateMachine initialized.");

    // 5. Initialize Mempool
    let mempool_config = MempoolConfig::default();
    let mempool = Arc::new(Mutex::new(Mempool::new(mempool_config)));
    tracing::info!("Mempool initialized with capacity: {}", mempool_config.max_transactions);

    // 6. Parse validator public keys from genesis and initialize ConsensusEngine
    let mut validator_public_keys = Vec::new();
    for (i, validator_hex) in genesis_data.validators.iter().enumerate() {
        tracing::info!("Parsing genesis validator {}: {}", i, validator_hex);
        let public_key_bytes = hex::decode(validator_hex)
            .map_err(|e| anyhow::anyhow!("Invalid validator public key hex: {}", e))?;
        if public_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Validator public key must be 32 bytes"));
        }
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
            .map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;
        let public_key = PublicKey(verifying_key);
        let derived_address = address_from_public_key(&public_key);
        tracing::info!("Genesis validator {} -> derived address: {}", i, hex::encode(derived_address.0));
        validator_public_keys.push(public_key);
    }

    // Load validator wallet from configured key file
    let validator_wallet = if let Some(validator_config) = &config.validator {
        if validator_config.enabled {
            tracing::info!("Loading validator key from: {}", validator_config.private_key_path);
            rustchain::wallet::Wallet::load_from_file(&validator_config.private_key_path)
                .map_err(|e| anyhow::anyhow!("Failed to load validator key: {}", e))?
        } else {
            tracing::info!("Validator mode disabled, creating dummy wallet");
            rustchain::wallet::Wallet::new()
        }
    } else {
        tracing::info!("No validator configuration, creating dummy wallet");
        rustchain::wallet::Wallet::new()
    };
    let consensus_engine = Arc::new(Mutex::new(ConsensusEngine::new(validator_public_keys.clone())));
    tracing::info!(
        "ConsensusEngine initialized with {} validator(s). Our validator address: {}", 
        validator_public_keys.len(),
        address_from_public_key(validator_wallet.public_key())
    );

    // 5. Initialize NetworkConfig
    let network_config = NetworkConfig::default();
    tracing::info!("NetworkConfig: {:?}", network_config);

    // 6. Generate Node Identity (Keypair)
    let local_keypair = identity::Keypair::generate_ed25519();
    let local_peer_id = Libp2pPeerId::from(local_keypair.public());
    tracing::info!("Generated local Peer ID: {}", local_peer_id);

    // 7. Create MPSC channel for incoming network messages
    let (incoming_message_sender, mut incoming_message_receiver) = mpsc::channel::<NetworkMessage>(128);

    // 8. Instantiate NetworkService
    tracing::info!("Initializing NetworkService...");
    let (network_service, network_command_sender) = 
        NetworkService::new(network_config.clone(), local_keypair, incoming_message_sender).await
        .map_err(|e| anyhow::anyhow!("Failed to create NetworkService: {}", e))?;
    tracing::info!("NetworkService initialized.");

    // Clone network service for block broadcasting
    let network_service_handle = network_service.command_sender();

    // 9. Spawn NetworkService::run() as a Tokio task
    tokio::spawn(network_service.run());

    // 10. Initial chain synchronization - request missing blocks from peers
    let sync_storage = storage.clone();
    let sync_network_sender = network_command_sender.clone();
    
    tokio::spawn(async move {
        // Wait a bit for network to connect to peers
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        tracing::info!("Starting initial chain synchronization...");
        
        // Get our current chain tip
        let storage_lock = sync_storage.lock().await;
        let (current_tip_hash, current_height) = match storage_lock.get_chain_tip() {
            Ok(Some((hash, height))) => {
                tracing::info!("Current chain height: {}", height);
                (hash, height)
            }
            Ok(None) => {
                tracing::info!("Empty chain, requesting blocks from height 1");
                (Hash([0u8; 32]), 0)
            }
            Err(e) => {
                tracing::error!("Failed to get chain tip for sync: {}", e);
                return;
            }
        };
        drop(storage_lock);
        
        // Request blocks starting from our next block
        let sync_request = NetworkMessage::SyncRequest {
            from_height: current_height + 1,
            to_hash: None, // Request all available blocks
        };
        
        // Broadcast sync request to peers
        if let Err(e) = sync_network_sender.send(rustchain::networking::NetworkCommand::BroadcastMessage {
            topic: rustchain::networking::Topic::new("sync"),
            message: sync_request,
        }).await {
            tracing::error!("Failed to send initial sync request: {}", e);
        } else {
            tracing::info!("Sent initial sync request for blocks starting from height {}", current_height + 1);
        }
        
        // Note: Responses will be handled by the message handler above
    });

    // Clone Arcs for the message handling task
    let consensus_engine_clone = consensus_engine.clone();
    let state_machine_clone = state_machine.clone();
    let storage_clone = storage.clone();
    let mempool_clone = mempool.clone();
    let network_command_sender_clone = network_command_sender.clone();

    // 11. Task to handle incoming messages from the NetworkService
    tokio::spawn(async move {
        tracing::info!("Incoming message handler task started.");
        while let Some(message) = incoming_message_receiver.recv().await {
            match message {
                NetworkMessage::NewTransaction(tx) => {
                    tracing::info!("Received NewTransaction: {}", tx.id().unwrap());
                    
                    // Add transaction to mempool
                    let mut mempool_lock = mempool_clone.lock().await;
                    match mempool_lock.add_transaction(tx) {
                        Ok(tx_hash) => {
                            tracing::info!("Transaction {} added to mempool", tx_hash);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to add transaction to mempool: {}", e);
                        }
                    }
                }
                NetworkMessage::NewBlock(block) => {
                    tracing::info!("Received NewBlock: height {}, hash {}", 
                        block.header.block_number.0, 
                        block.header.calculate_hash().unwrap_or_default()
                    );

                    // Validate block through consensus
                    let consensus_engine = consensus_engine_clone.lock().await;
                    if let Err(e) = consensus_engine.validate_block(&block) {
                        tracing::warn!("Invalid block received: {}", e);
                        continue;
                    }
                    drop(consensus_engine);

                    // Apply block to state machine
                    let mut state_machine = state_machine_clone.lock().await;
                    if let Err(e) = state_machine.apply_block(&block) {
                        tracing::warn!("Failed to apply block to state machine: {}", e);
                        continue;
                    }

                    // Remove included transactions from mempool
                    let tx_hashes: Vec<Hash> = block.transactions.iter()
                        .filter_map(|tx| tx.id().ok())
                        .collect();
                    let mut mempool_lock = mempool_clone.lock().await;
                    mempool_lock.remove_transactions(&tx_hashes);
                    drop(mempool_lock);

                    // Persist block and updated state to storage
                    let storage = storage_clone.lock().await;
                    if let Err(e) = storage.commit_block(&block, &state_machine.world_state) {
                        tracing::error!("Failed to commit block to storage: {}", e);
                        continue;
                    }

                    tracing::info!("Successfully processed and committed new block: height {}", block.header.block_number.0);
                }
                NetworkMessage::SyncRequest { from_height, to_hash } => {
                    tracing::info!("Received SyncRequest: from_height {}, to_hash {:?}", from_height, to_hash);
                    
                    // Respond with blocks from our storage
                    let storage_lock = storage_clone.lock().await;
                    let (current_tip_hash, current_height) = match storage_lock.get_chain_tip() {
                        Ok(Some((hash, height))) => (hash, height),
                        Ok(None) => {
                            tracing::warn!("Cannot respond to sync request: no chain tip");
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Failed to get chain tip for sync response: {}", e);
                            continue;
                        }
                    };
                    
                    let mut blocks_to_send = Vec::new();
                    let max_blocks = 50; // Limit blocks per response
                    let end_height = std::cmp::min(current_height, from_height + max_blocks - 1);
                    
                    // For now, we'll implement a simple approach - just send current tip if requested
                    // TODO: Implement proper height-based block retrieval
                    if from_height <= current_height {
                        if let Ok(Some(block)) = storage_lock.get_block(&current_tip_hash) {
                            blocks_to_send.push(block);
                        }
                    }
                    drop(storage_lock);
                    
                    // Send response
                    let response_message = if blocks_to_send.is_empty() {
                        NetworkMessage::SyncResponseNoBlocks
                    } else {
                        NetworkMessage::SyncResponseBlocks { blocks: blocks_to_send }
                    };
                    
                    // Broadcast the response (in a real implementation, this would be sent to specific peer)
                    if let Err(e) = network_command_sender_clone.send(rustchain::networking::NetworkCommand::BroadcastMessage {
                        topic: rustchain::networking::Topic::new("sync"),
                        message: response_message,
                    }).await {
                        tracing::error!("Failed to send sync response: {}", e);
                    }
                }
                NetworkMessage::SyncResponseBlocks { blocks } => {
                    tracing::info!("Received SyncResponseBlocks with {} blocks", blocks.len());
                    
                    // Process each block in order
                    for block in blocks {
                        // Validate block through consensus
                        let consensus_engine = consensus_engine_clone.lock().await;
                        if let Err(e) = consensus_engine.validate_block(&block) {
                            tracing::warn!("Invalid block in sync response: {}", e);
                            drop(consensus_engine);
                            continue;
                        }
                        drop(consensus_engine);

                        // Apply block to state machine
                        let mut state_machine = state_machine_clone.lock().await;
                        if let Err(e) = state_machine.apply_block(&block) {
                            tracing::warn!("Failed to apply synced block to state machine: {}", e);
                            drop(state_machine);
                            continue;
                        }

                        // Persist block and updated state to storage
                        let storage = storage_clone.lock().await;
                        if let Err(e) = storage.commit_block(&block, &state_machine.world_state) {
                            tracing::error!("Failed to commit synced block to storage: {}", e);
                            drop(storage);
                            drop(state_machine);
                            continue;
                        }
                        drop(storage);
                        drop(state_machine);

                        tracing::info!("Successfully synced and committed block: height {}", block.header.block_number.0);
                    }
                }
                NetworkMessage::SyncResponseNoBlocks => {
                    tracing::info!("Received SyncResponseNoBlocks - peer has no blocks to send");
                    // Handle case where peer doesn't have the requested blocks
                }
            }
        }
    });

    // 12. Block production task - only runs if this node is a validator
    let mempool_producer = mempool.clone();
    let consensus_producer = consensus_engine.clone();
    let state_producer = state_machine.clone();
    let storage_producer = storage.clone();
    let network_sender = network_command_sender.clone();
    let validator_wallet_clone = validator_wallet;
    
    // Extract config values before moving into async task
    let block_interval = config.consensus.block_interval;
    let max_txs_per_block = config.consensus.max_txs_per_block;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(block_interval));
        
        loop {
            interval.tick().await;
            tracing::info!("Block production timer tick");
            
            // Check if it's our turn to propose
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Get current blockchain state
            let storage_lock = storage_producer.lock().await;
            let (current_tip_hash, current_height) = match storage_lock.get_chain_tip() {
                Ok(Some((hash, height))) => (hash, height),
                Ok(None) => {
                    // Genesis case - start with height 0
                    (Hash([0u8; 32]), 0)
                }
                Err(e) => {
                    tracing::error!("Failed to get chain tip: {}", e);
                    continue;
                }
            };
            drop(storage_lock);
            
            let next_height = BlockHeight(current_height + 1);
            
            // Check with consensus engine if we should propose
            let consensus_lock = consensus_producer.lock().await;
            let expected_proposer = match consensus_lock.get_proposer(next_height) {
                Ok(proposer) => proposer,
                Err(e) => {
                    tracing::debug!("Failed to get proposer for height {}: {}", next_height.0, e);
                    drop(consensus_lock);
                    continue;
                }
            };
            
            let our_address = address_from_public_key(validator_wallet_clone.public_key());
            let expected_address = address_from_public_key(expected_proposer);
            
            if our_address != expected_address {
                tracing::info!("Not our turn to propose. Expected: {}, We are: {}", hex::encode(expected_address.0), hex::encode(our_address.0));
                drop(consensus_lock);
                continue;
            }
            drop(consensus_lock);
            
            tracing::info!("Our turn to propose block at height {}", next_height.0);
            
            // Collect transactions from mempool
            let mempool_lock = mempool_producer.lock().await;
            let transactions = mempool_lock.get_pending_transactions(max_txs_per_block);
            let num_txs = transactions.len();
            drop(mempool_lock);
            
            tracing::info!("Collected {} transactions for new block", num_txs);
            
            // Calculate merkle root
            let tx_root = match calculate_merkle_root(&transactions) {
                Ok(root) => root,
                Err(e) => {
                    tracing::error!("Failed to calculate merkle root: {}", e);
                    continue;
                }
            };
            
            // Create block header (without signature first)
            let mut block_header = BlockHeader {
                parent_hash: current_tip_hash,
                block_number: next_height,
                timestamp: Timestamp(current_time),
                tx_root,
                validator: our_address,
                signature: Signature(vec![0; 64]), // Placeholder
            };
            
            // Calculate header hash and sign it
            let header_hash = match block_header.calculate_hash() {
                Ok(hash) => hash,
                Err(e) => {
                    tracing::error!("Failed to calculate header hash: {}", e);
                    continue;
                }
            };
            
            let signature = match validator_wallet_clone.sign(header_hash.as_ref()) {
                Ok(sig) => sig,
                Err(e) => {
                    tracing::error!("Failed to sign block header: {}", e);
                    continue;
                }
            };
            
            // Update header with real signature
            block_header.signature = signature;
            
            // Create the complete block
            let new_block = Block {
                header: block_header,
                transactions,
            };
            
            tracing::info!("Produced new block: height {}, txs {}, hash {}", 
                new_block.header.block_number.0,
                new_block.transactions.len(),
                new_block.header.calculate_hash().unwrap_or_default()
            );
            
            // Apply block locally first (optimistic)
            let mut state_lock = state_producer.lock().await;
            if let Err(e) = state_lock.apply_block(&new_block) {
                tracing::error!("Failed to apply our own block to state machine: {}", e);
                continue;
            }
            
            // Remove transactions from mempool
            let tx_hashes: Vec<Hash> = new_block.transactions.iter()
                .filter_map(|tx| tx.id().ok())
                .collect();
            let mut mempool_lock = mempool_producer.lock().await;
            mempool_lock.remove_transactions(&tx_hashes);
            drop(mempool_lock);
            
            // Persist the block
            let storage_lock = storage_producer.lock().await;
            if let Err(e) = storage_lock.commit_block(&new_block, &state_lock.world_state) {
                tracing::error!("Failed to commit our own block to storage: {}", e);
                continue;
            }
            drop(storage_lock);
            drop(state_lock);
            
            // Broadcast the block to peers
            let broadcast_command = rustchain::networking::NetworkCommand::BroadcastBlock(new_block.clone());
            if let Err(e) = network_sender.send(broadcast_command).await {
                tracing::error!("Failed to send broadcast block command: {}", e);
            } else {
                tracing::info!("Successfully sent block broadcast command to network");
            }
        }
    });

    tracing::info!("RustChain Node is running. Press Ctrl-C to stop.");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Ctrl-C received, shutting down node...");

    Ok(())
}
