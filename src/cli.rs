use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bunker")]
#[command(about = "Dead simple, secure password management")]
#[command(version = "1.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Vault to operate on
    #[arg(long, global = true)]
    pub vault: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        /// Name of the vault
        name: String,
        /// Run in non-interactive mode
        #[arg(long)]
        non_interactive: bool,
    },

    /// Add a new password
    Add {
        /// Entry key/name
        key: String,
        /// Password value (will prompt if not provided)
        #[arg(long)]
        value: Option<String>,
        /// Store as a note instead of password
        #[arg(long)]
        note: bool,
        /// Read content from file
        #[arg(long)]
        file: Option<PathBuf>,
    },

    /// Get a password
    Get {
        /// Entry key/name
        key: String,
        /// Copy to clipboard instead of printing
        #[arg(short, long)]
        copy: bool,
        /// Clipboard timeout in seconds
        #[arg(long, default_value = "45")]
        timeout: u64,
    },

    /// Edit an existing password
    Edit {
        /// Entry key/name  
        key: String,
        /// New password value (will prompt if not provided)
        #[arg(long)]
        value: Option<String>,
    },

    /// Remove a password
    Remove {
        /// Entry key/name
        key: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List all passwords
    List {
        /// Show as tree structure
        #[arg(short, long)]
        tree: bool,
    },

    /// Search passwords
    Search {
        /// Search query (optional for interactive search)
        query: Option<String>,
    },

    /// Generate a secure password
    Generate {
        /// Password length
        #[arg(short, long, default_value = "20")]
        length: usize,
        /// Include uppercase letters
        #[arg(long)]
        uppercase: bool,
        /// Include lowercase letters  
        #[arg(long)]
        lowercase: bool,
        /// Include numbers
        #[arg(long)]
        numbers: bool,
        /// Include symbols
        #[arg(long)]
        symbols: bool,
        /// Exclude ambiguous characters
        #[arg(long)]
        no_ambiguous: bool,
        /// Custom character set
        #[arg(long)]
        charset: Option<String>,
        /// Copy to clipboard
        #[arg(short, long)]
        copy: bool,
    },

    /// Copy password to clipboard
    Copy {
        /// Entry key/name
        key: String,
        /// Clipboard timeout in seconds
        #[arg(long, default_value = "45")]
        timeout: u64,
    },

    /// Peek at password (show masked)
    Peek {
        /// Entry key/name
        key: String,
        /// Number of characters to show
        #[arg(long, default_value = "3")]
        chars: usize,
    },

    /// Move/rename a password
    #[command(name = "mv")]
    Move {
        /// Source key
        from: String,
        /// Target key
        to: String,
    },

    /// Execute command with password as argument or environment variable
    Exec {
        /// Command to execute
        #[arg(last = true)]
        command: Vec<String>,
        /// Entry key/name
        #[arg(short, long)]
        key: String,
        /// Use as environment variable
        #[arg(short, long)]
        env: Option<String>,
    },

    /// Export vault data
    Export {
        /// Export format (json, csv, pass)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include metadata
        #[arg(long)]
        metadata: bool,
    },

    /// Import passwords from file
    Import {
        /// Input file
        file: PathBuf,
        /// Import format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Overwrite existing entries
        #[arg(long)]
        overwrite: bool,
    },

    /// Search with grep-like patterns
    Grep {
        /// Search pattern (regex)
        pattern: String,
        /// Case insensitive search
        #[arg(short, long)]
        ignore_case: bool,
    },

    /// Version control operations
    Git {
        #[command(subcommand)]
        action: GitAction,
    },

    /// Vault management
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },

    /// Lock the vault
    Lock,

    /// Unlock the vault
    Unlock {
        /// Session duration in hours
        #[arg(long, default_value = "24")]
        duration: u64,
    },

    /// Show vault status
    Status,

    /// Backup vault
    Backup {
        /// Backup destination
        destination: Option<PathBuf>,
    },

    /// Restore from backup
    Restore {
        /// Backup file to restore from
        backup: PathBuf,
        /// Target vault name
        #[arg(long)]
        vault_name: Option<String>,
    },

    /// Show command history  
    History {
        /// Entry key (optional)
        key: Option<String>,
        /// Limit number of entries
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Export environment variable
    Env {
        /// Entry key/name
        key: String,
        /// Environment variable name
        #[arg(long)]
        var: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GitAction {
    /// Sync with remote
    Sync,
    /// Pull from remote
    Pull,
    /// Show git status
    Status,
    /// Restore from commit
    Restore {
        /// Commit hash
        commit: String,
        /// Entry key (optional)
        key: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum VaultAction {
    /// Create a new vault
    Create {
        /// Vault name
        name: String,
    },
    /// Switch to a vault
    Use {
        /// Vault name
        name: String,
    },
    /// List all vaults
    List,
    /// Delete a vault
    Delete {
        /// Vault name
        name: String,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Export vault (encrypted)
    Export {
        /// Export password
        password: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Import vault (encrypted)
    Import {
        /// Import file
        file: PathBuf,
        /// Import password
        password: String,
        /// Target vault name
        name: String,
    },
}

pub struct CliDisplay;

impl CliDisplay {
    pub fn print_banner() {
        println!(
            "{} {} {}",
            "🔐".blue(),
            "BUNKER".cyan().bold(),
            "Secure Password Manager".white()
        );
        println!("{}", "Fast • Secure • Simple".dimmed());
        println!();
    }

    pub fn print_welcome() {
        println!("{}", "Welcome to Bunker!".green().bold());
        println!("Your passwords are locked down tight.\n");
    }

    pub fn print_init_success(vault_name: &str) {
        println!(
            "{} Vault '{}' initialized successfully!",
            "✓".green().bold(),
            vault_name.cyan()
        );
        println!("\nNext steps:");
        println!(
            "  {} Add your first password: {}",
            "→".blue(),
            format!("bunker add <key>").white().bold()
        );
        println!(
            "  {} List passwords: {}",
            "→".blue(),
            format!("bunker list").white().bold()
        );
        println!(
            "  {} Copy password: {}",
            "→".blue(),
            format!("bunker copy <key>").white().bold()
        );
    }

    pub fn print_entry_added(key: &str) {
        println!(
            "{} Password '{}' added successfully",
            "✓".green().bold(),
            key.cyan()
        );
    }

    pub fn print_entry_removed(key: &str) {
        println!("{} Password '{}' removed", "✗".red().bold(), key.cyan());
    }

    pub fn print_entry_copied(key: &str, timeout: u64) {
        println!(
            "{} Password '{}' copied to clipboard",
            "📋".green().bold(),
            key.cyan()
        );
        if timeout > 0 {
            println!(
                "Clipboard will clear in {} seconds",
                timeout.to_string().yellow()
            );
        }
    }

    pub fn print_masked_password(key: &str, masked: &str) {
        println!("{}: {}", key.cyan(), masked);
    }

    pub fn print_session_status(active: bool, vault: &str) {
        if active {
            println!(
                "{} Session active for vault '{}'",
                "🔓".green(),
                vault.cyan()
            );
        } else {
            println!(
                "{} No active session for vault '{}'",
                "🔒".yellow(),
                vault.cyan()
            );
        }
    }

    pub fn print_sync_success() {
        println!("{} Sync completed successfully", "✓".green().bold());
    }

    pub fn print_export_success(path: Option<&str>) {
        if let Some(p) = path {
            println!("{} Vault exported to: {}", "✓".green().bold(), p.cyan());
        } else {
            println!("{} Vault exported successfully", "✓".green().bold());
        }
    }

    pub fn print_import_success(vault_name: &str) {
        println!(
            "{} Vault '{}' imported successfully",
            "✓".green().bold(),
            vault_name.cyan()
        );
    }

    pub fn print_qr_code(code: &str) {
        println!("\n{}", "QR Code:".yellow().bold());
        println!("{}", code);
    }
}

// Add these implementations for compatibility
impl Cli {
    pub fn print_banner() {
        CliDisplay::print_banner();
    }

    pub fn print_welcome() {
        CliDisplay::print_welcome();
    }

    pub fn print_init_success(vault_name: &str) {
        CliDisplay::print_init_success(vault_name);
    }

    pub fn print_entry_added(key: &str) {
        CliDisplay::print_entry_added(key);
    }

    pub fn print_entry_removed(key: &str) {
        CliDisplay::print_entry_removed(key);
    }

    pub fn print_entry_copied(key: &str, timeout: u64) {
        CliDisplay::print_entry_copied(key, timeout);
    }

    pub fn print_masked_password(key: &str, masked: &str) {
        CliDisplay::print_masked_password(key, masked);
    }

    pub fn print_session_status(active: bool, vault: &str) {
        CliDisplay::print_session_status(active, vault);
    }

    pub fn print_sync_success() {
        CliDisplay::print_sync_success();
    }

    pub fn print_export_success(path: Option<&str>) {
        CliDisplay::print_export_success(path);
    }

    pub fn print_import_success(vault_name: &str) {
        CliDisplay::print_import_success(vault_name);
    }

    pub fn print_qr_code(code: &str) {
        CliDisplay::print_qr_code(code);
    }
}
