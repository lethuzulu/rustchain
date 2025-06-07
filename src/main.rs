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
use std::sync::Arc;
use tokio::sync::Mutex;

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
    // Example: #[clap(long, short, default_value = "/ip4/0.0.0.0/tcp/0")]
    // pub listen_address: String, // We'll use NetworkConfig::default for now
    // Example: #[clap(long)]
    // pub bootstrap_peers: Vec<String>,
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

    // 3. Initialize ConsensusEngine
    let validator_wallet = rustchain::wallet::Wallet::new();
    let validators = vec![*validator_wallet.public_key()];
    let consensus_engine = Arc::new(Mutex::new(ConsensusEngine::new(validators.clone())));
    tracing::info!("ConsensusEngine initialized with {} validator(s).", validators.len());

    // 4. Initialize NetworkConfig
    let network_config = NetworkConfig::default();
    tracing::info!("NetworkConfig: {:?}", network_config);

    // 5. Generate Node Identity (Keypair)
    let local_keypair = identity::Keypair::generate_ed25519();
    let local_peer_id = Libp2pPeerId::from(local_keypair.public());
    tracing::info!("Generated local Peer ID: {}", local_peer_id);

    // 6. Create MPSC channel for incoming network messages
    let (incoming_message_sender, mut incoming_message_receiver) = mpsc::channel::<NetworkMessage>(128);

    // 7. Instantiate NetworkService
    tracing::info!("Initializing NetworkService...");
    let (network_service, network_command_sender) = 
        NetworkService::new(network_config.clone(), local_keypair, incoming_message_sender).await
        .map_err(|e| anyhow::anyhow!("Failed to create NetworkService: {}", e))?;
    tracing::info!("NetworkService initialized.");

    // 8. Spawn NetworkService::run() as a Tokio task
    tokio::spawn(network_service.run());

    // Clone Arcs for the message handling task
    let consensus_engine_clone = consensus_engine.clone();
    let state_machine_clone = state_machine.clone();
    let storage_clone = storage.clone();

    // 9. Task to handle incoming messages from the NetworkService
    tokio::spawn(async move {
        tracing::info!("Incoming message handler task started.");
        while let Some(message) = incoming_message_receiver.recv().await {
            match message {
                NetworkMessage::NewTransaction(tx) => {
                    tracing::info!("Received NewTransaction: {}", tx.id().unwrap());
                    // TODO: Pass to Mempool
                }
                NetworkMessage::NewBlock(block) => {
                    tracing::info!("Received NewBlock: {}", block.header.block_number);

                    let consensus_engine = consensus_engine_clone.lock().await;
                    if let Err(e) = consensus_engine.validate_block(&block) {
                        tracing::warn!("Invalid block received: {}", e);
                        continue;
                    }

                    let mut state_machine = state_machine_clone.lock().await;
                    if let Err(e) = state_machine.apply_block(&block) {
                        tracing::warn!("Failed to apply block to state machine: {}", e);
                        continue;
                    }

                    let storage = storage_clone.lock().await;
                    if let Err(e) = storage.commit_block(&block, &state_machine.world_state) {
                        tracing::error!("Failed to commit block to storage: {}", e);
                    }

                    tracing::info!("Successfully processed and committed new block: {}", block.header.block_number);
                }
            }
        }
    });

    tracing::info!("RustChain Node is running. Press Ctrl-C to stop.");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Ctrl-C received, shutting down node...");

    Ok(())
}
