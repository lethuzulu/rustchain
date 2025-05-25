use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rustchain::wallet::Wallet;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage wallets (generate, show)
    Wallet(WalletArgs),
    // Other top-level commands like 'node', 'explorer'
}

#[derive(Parser, Debug)]
struct WalletArgs {
    #[clap(subcommand)]
    action: WalletAction,
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
    // 'send' subcommand will be added later
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Wallet(wallet_args) => {
            match &wallet_args.action {
                WalletAction::Generate { keyfile } => {
                    handle_generate_wallet(keyfile)?;
                }
                WalletAction::Show { keyfile } => {
                    handle_show_wallet(keyfile)?;
                }
            }
        }
        // Handle other commands like starting a node, etc.
    }
    Ok(())
}

// --- Wallet Command Handlers ---

const DEFAULT_KEY_FILE: &str = "default_wallet.key";

fn handle_generate_wallet(keyfile_opt: &Option<PathBuf>) -> anyhow::Result<()> {
    let wallet = Wallet::new();
    let keyfile_path: PathBuf = keyfile_opt.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_FILE));
    
    wallet.save_to_file(keyfile_path.to_str().unwrap_or(DEFAULT_KEY_FILE))?;
    println!("Generated new wallet and saved to: {}", keyfile_path.display());
    println!("  Address: {}", wallet.address());
    println!("  Public Key: {}", wallet.public_key());
    // For security, we don't print the private key here.
    Ok(())
}

fn handle_show_wallet(keyfile_opt: &Option<PathBuf>) -> anyhow::Result<()> {
    let keyfile_path: PathBuf = keyfile_opt.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_KEY_FILE));

    if !keyfile_path.exists() {
        eprintln!("Error: Key file not found at path: {}", keyfile_path.display());
        eprintln!("Please generate a wallet first using 'wallet generate' or provide a valid --keyfile path.");
        return Ok(()); // Or return an error if preferred for stricter handling
    }

    let wallet = Wallet::load_from_file(keyfile_path.to_str().unwrap_or(DEFAULT_KEY_FILE))?;
    println!("Wallet details from: {}", keyfile_path.display());
    println!("  Address: {}", wallet.address());
    println!("  Public Key: {}", wallet.public_key());
    Ok(())
}

// TODO:
// - Implement `wallet send` command and its handler
// - Add actual transaction creation and signing logic
// - Integrate with a node for balance queries and transaction submission for `send`
