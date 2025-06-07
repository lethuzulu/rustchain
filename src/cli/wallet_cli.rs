use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rustchain::wallet::Wallet; // Changed from rustchain::wallet
use rustchain::types::{Address, Nonce};
use bincode;
use anyhow;
use hex; // Added hex import

// Main CLI structure if this module handles the entire `rustchain` command.
// If `main.rs` has its own top-level Commands (e.g. for `node` vs `wallet`),
// then this Cli/Commands might be simplified or adjusted.
#[derive(Parser, Debug)]
#[clap(author, version, about = "RustChain CLI utility", long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage wallets (generate, show, send)
    #[clap(name = "wallet")] // Use a specific name for the subcommand if desired
    Wallet(WalletCliArgs),
    // Add other top-level commands like `node` here if this is the main CLI parser
}

#[derive(Parser, Debug)]
pub struct WalletCliArgs { // This struct now holds the sub-actions for the `wallet` command
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
    nonce_val: u64, 
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
    
    let nonce = Nonce(nonce_val);

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

    let config = bincode::config::standard();
    let serialized_tx = bincode::encode_to_vec(&transaction, config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize final transaction: {}", e))?;
    
    println!("\nSerialized Signed Transaction (hex for broadcast/storage):");
    println!("{}", hex::encode(serialized_tx));

    Ok(())
}

/// Main entry point for wallet CLI commands
pub fn run_wallet_cli(cli_args: WalletCliArgs) -> anyhow::Result<()> {
    match &cli_args.action {
        WalletAction::Generate { keyfile } => {
            handle_generate_wallet(keyfile)?;
        }
        WalletAction::Show { keyfile } => {
            handle_show_wallet(keyfile)?;
        }
        WalletAction::Send { to, amount, nonce, keyfile } => {
            handle_send_transaction(to, *amount, *nonce, keyfile)?;
        }
    }
    Ok(())
} 