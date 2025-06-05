use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rustchain::wallet::Wallet;
use rustchain::types::{Address, Nonce}; // For parsing arguments
use bincode; // For serializing the final transaction for display

// Networking related imports
use rustchain::networking::{NetworkService, NetworkConfig, NetworkMessage, Libp2pPeerId};
use libp2p::identity;
use tokio::sync::mpsc;
use tracing_subscriber::fmt::format::FmtSpan;

// This brings the cli module into scope, which exports wallet_cli.
// wallet_cli, in turn, exports its own Cli and Commands structs.
mod cli;

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

#[derive(Parser, Debug)]
struct WalletArgs {
    #[clap(subcommand)]
    action: WalletAction,
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

#[derive(Subcommand, Debug)]
enum WalletAction {
    /// Generate a new wallet and save the key to a file
    Generate {
        /// Optional: Path to save the generated key file
        #[clap(short, long, value_parser)]
        keyfile: Option<PathBuf>,
    },
    /// Show wallet address and public key from a key file
    Show {
        /// Optional: Path to the key file to load
        #[clap(short, long, value_parser)]
        keyfile: Option<PathBuf>,
    },
    /// Create and sign a transaction, then print it (serialized)
    Send {
        /// Recipient's address (hex string, e.g., 0x...)
        #[clap(long, value_parser = parse_address)]
        to: Address,
        /// Amount to send
        #[clap(long)]
        amount: u64,
        /// Transaction nonce
        #[clap(long)]
        nonce: u64, // Will be wrapped into Nonce type
        /// Optional: Path to the key file to use for sending
        #[clap(short, long, value_parser)]
        keyfile: Option<PathBuf>,
    },
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

    // 1. Initialize NetworkConfig (using default for now)
    let network_config = NetworkConfig::default();
    tracing::info!("NetworkConfig: {:?}", network_config);

    // 2. Generate Node Identity (Keypair)
    let local_keypair = identity::Keypair::generate_ed25519();
    let local_peer_id = Libp2pPeerId::from(local_keypair.public());
    tracing::info!("Generated local Peer ID: {}", local_peer_id);

    // 3. Create MPSC channel for incoming network messages
    // The NetworkService will send messages here, and our node logic will receive them.
    let (incoming_message_sender, mut incoming_message_receiver) = mpsc::channel::<NetworkMessage>(128); // Buffer size 128

    // 4. Instantiate NetworkService
    tracing::info!("Initializing NetworkService...");
    let (network_service, network_command_sender) = 
        NetworkService::new(network_config.clone(), local_keypair, incoming_message_sender).await
        .map_err(|e| anyhow::anyhow!("Failed to create NetworkService: {}", e))?;
    tracing::info!("NetworkService initialized.");

    // 5. Spawn NetworkService::run() as a Tokio task
    tracing::info!("Spawning NetworkService event loop...");
    tokio::spawn(network_service.run()); // This consumes network_service
    tracing::info!("NetworkService event loop spawned.");

    // 6. Task to handle incoming messages from the NetworkService
    tokio::spawn(async move {
        tracing::info!("Incoming message handler task started.");
        while let Some(message) = incoming_message_receiver.recv().await {
            match message {
                NetworkMessage::NewTransaction(tx) => {
                    tracing::info!("Received NewTransaction from network: ID: {}, Sender: {}, Amount: {}", 
                                   tx.id().unwrap_or_default(), tx.sender, tx.amount);
                    // TODO: Pass to Mempool
                }
                NetworkMessage::NewBlock(block) => {
                    tracing::info!("Received NewBlock from network: Number: {}, Validator: {}, Tx Count: {}", 
                                   block.header.block_number, block.header.validator, block.transactions.len());
                    // TODO: Pass to Consensus/Block Validation
                }
            }
        }
        tracing::info!("Incoming message handler task stopped.");
    });

    // TODO: The node will need a way to gracefully shut down.
    // For now, it runs until Ctrl-C if the main function can await something indefinitely.
    // Example: A simple way to keep main alive for testing networking
    tracing::info!("RustChain Node is running. Press Ctrl-C to stop.");
    // This will keep the main thread alive, allowing background tasks to run.
    // In a real node, this would be a more sophisticated shutdown mechanism.
    tokio::signal::ctrl_c().await?;
    tracing::info!("Ctrl-C received, shutting down node...");

    Ok(())
}

// --- Wallet Command Handlers ---

const DEFAULT_KEY_FILE: &str = "default_wallet.key";

fn handle_generate_wallet(keyfile_opt: &Option<PathBuf>) -> anyhow::Result<()> {
    let wallet = Wallet::new();
    let keyfile_path: PathBuf = keyfile_opt.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_FILE));
    
    wallet.save_to_file(keyfile_path.to_str().unwrap_or(DEFAULT_KEY_FILE))
        .map_err(|e| anyhow::anyhow!("Failed to save wallet to {}: {}", keyfile_path.display(), e))?;
    println!("Generated new wallet and saved to: {}", keyfile_path.display());
    println!("  Address: {}", wallet.address());
    println!("  Public Key: {}", wallet.public_key());
    Ok(())
}

fn handle_show_wallet(keyfile_opt: &Option<PathBuf>) -> anyhow::Result<()> {
    let keyfile_path: PathBuf = keyfile_opt.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_FILE));

    if !keyfile_path.exists() {
        return Err(anyhow::anyhow!(
            "Error: Key file not found at path: {}. Please generate a wallet or provide a valid --keyfile path.", 
            keyfile_path.display()
        ));
    }

    let wallet = Wallet::load_from_file(keyfile_path.to_str().unwrap_or(DEFAULT_KEY_FILE))
        .map_err(|e| anyhow::anyhow!("Failed to load wallet from {}: {}", keyfile_path.display(), e))?;
    println!("Wallet details from: {}", keyfile_path.display());
    println!("  Address: {}", wallet.address());
    println!("  Public Key: {}", wallet.public_key());
    Ok(())
}

fn handle_send_transaction(
    to: &Address, 
    amount: u64, 
    nonce: Nonce, 
    keyfile_opt: &Option<PathBuf>
) -> anyhow::Result<()> {
    let keyfile_path = keyfile_opt.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_FILE));

    if !keyfile_path.exists() {
        return Err(anyhow::anyhow!(
            "Error: Key file not found at path: {}. Cannot send transaction.", 
            keyfile_path.display()
        ));
    }

    println!("Loading wallet from: {}", keyfile_path.display());
    let wallet = Wallet::load_from_file(keyfile_path.to_str().unwrap_or(DEFAULT_KEY_FILE))
        .map_err(|e| anyhow::anyhow!("Failed to load wallet for sending: {}", e))?;
    
    println!("Creating transaction...");
    println!("  Sender (from keyfile): {}", wallet.address());
    println!("  Recipient: {}", to);
    println!("  Amount: {}", amount);
    println!("  Nonce: {}", nonce.0);

    let transaction = wallet.create_signed_transaction(*to, amount, nonce)
        .map_err(|e| anyhow::anyhow!("Failed to create signed transaction: {}", e))?;

    println!("\nSigned Transaction Details:");
    println!("  Sender: {}", transaction.sender);
    println!("  Recipient: {}", transaction.recipient);
    println!("  Amount: {}", transaction.amount);
    println!("  Nonce: {}", transaction.nonce.0);
    println!("  Signature: {}", transaction.signature);

    // Serialize the full transaction for output (e.g., to be broadcasted later)
    let config = bincode::config::standard();
    let serialized_tx = bincode::encode_to_vec(&transaction, config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize final transaction: {}", e))?;
    
    println!("\nSerialized Signed Transaction (hex for broadcast/storage):");
    println!("{}", hex::encode(serialized_tx));

    Ok(())
}

// TODO:
// - Implement `wallet send` command and its handler
// - Add actual transaction creation and signing logic
// - Integrate with a node for balance queries and transaction submission for `send`
