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
use rustchain::types::{BlockHeight, Hash, Signature, Timestamp, address_from_public_key};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::Signer;

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
    /// Block production interval in seconds (default: 5)
    #[clap(long, default_value = "5")]
    pub block_interval: u64,
    
    /// Maximum transactions per block (default: 10)
    #[clap(long, default_value = "10")]
    pub max_txs_per_block: usize,
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
            run_node(node_args).await?;
        }
    }

    Ok(())
}

async fn run_node(node_args: NodeArgs) -> anyhow::Result<()> {
    tracing::info!("Starting RustChain node...");

    // 1. Initialize Storage
    let db_path = "rustchain_db";
    let storage = Arc::new(Mutex::new(
        Storage::new(db_path).map_err(|e| anyhow::anyhow!("Failed to initialize storage: {}", e))?,
    ));
    tracing::info!("Storage initialized at: {}", db_path);

    // 2. Initialize StateMachine
    let state_machine = Arc::new(Mutex::new(StateMachine::new()));
    tracing::info!("StateMachine initialized.");

    // 3. Initialize Mempool
    let mempool_config = MempoolConfig::default();
    let mempool = Arc::new(Mutex::new(Mempool::new(mempool_config)));
    tracing::info!("Mempool initialized with capacity: {}", mempool_config.max_transactions);

    // 4. Initialize ConsensusEngine and Validator Wallet
    let validator_wallet = rustchain::wallet::Wallet::new();
    let validators = vec![*validator_wallet.public_key()];
    let consensus_engine = Arc::new(Mutex::new(ConsensusEngine::new(validators.clone())));
    tracing::info!(
        "ConsensusEngine initialized with {} validator(s). Our validator address: {}", 
        validators.len(),
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

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(node_args.block_interval));
        
        loop {
            interval.tick().await;
            
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
                tracing::debug!("Not our turn to propose. Expected: {}, We are: {}", expected_address, our_address);
                drop(consensus_lock);
                continue;
            }
            drop(consensus_lock);
            
            tracing::info!("Our turn to propose block at height {}", next_height.0);
            
            // Collect transactions from mempool
            let mempool_lock = mempool_producer.lock().await;
            let transactions = mempool_lock.get_pending_transactions(node_args.max_txs_per_block);
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
