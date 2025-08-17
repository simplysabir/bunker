use anyhow::{Result, anyhow};
use colored::*;

use crate::cli::Cli;
use crate::config::Config;
use crate::git::Git;
use crate::storage::Storage;

pub async fn execute(vault: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let vault_name = vault.unwrap_or(config.default_vault.clone());
    let storage = Storage::new(Some(vault_name.clone()))?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault '{}' not initialized", vault_name));
    }

    let vault_config = storage.load_config()?;

    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!(
        "{}",
        "║         BUNKER VAULT STATUS          ║".cyan().bold()
    );
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!();

    // Vault info
    println!("{}:", "Vault".white().bold());
    println!("  Name: {}", vault_name.cyan());
    println!(
        "  Created: {}",
        vault_config.created_at.format("%Y-%m-%d %H:%M")
    );
    println!(
        "  Modified: {}",
        vault_config.last_modified.format("%Y-%m-%d %H:%M")
    );
    println!();

    // Encryption info
    println!("{}:", "Encryption".white().bold());
    println!("  Algorithm: {}", vault_config.encryption.algorithm);
    println!("  KDF: {}", vault_config.encryption.kdf);
    println!();

    // Session info
    let session_active = storage.load_session().is_ok();
    Cli::print_session_status(session_active, &vault_name);
    println!();

    // Statistics
    let entries = storage.list_entries()?;
    println!("{}:", "Statistics".white().bold());
    println!("  Passwords: {}", entries.len().to_string().green().bold());

    if !entries.is_empty() {
        // Count by directory
        let mut dirs = std::collections::HashMap::new();
        for entry in &entries {
            let dir = entry.split('/').next().unwrap_or("root");
            *dirs.entry(dir).or_insert(0) += 1;
        }

        let mut sorted_dirs: Vec<_> = dirs.iter().collect();
        sorted_dirs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

        println!("\n  Top directories:");
        for (dir, count) in sorted_dirs.iter().take(5) {
            println!("    {} ({})", dir.cyan(), count);
        }
    }
    println!();

    // Git status
    if Git::is_repo(storage.get_vault_path())? {
        println!("{}:", "Git".white().bold());

        if let Some(remote) = &vault_config.git_remote {
            println!("  Remote: {}", remote.cyan());
        } else {
            println!("  Remote: {}", "Not configured".yellow());
        }

        let changes = Git::status(storage.get_vault_path())?;
        if changes.is_empty() {
            println!("  Status: {}", "Clean".green());
        } else {
            println!("  Status: {} changes", changes.len().to_string().yellow());
            for change in changes.iter().take(3) {
                println!("    {}", change);
            }
            if changes.len() > 3 {
                println!("    ... and {} more", changes.len() - 3);
            }
        }
    } else {
        println!("{}:", "Git".white().bold());
        println!("  Status: {}", "Not initialized".yellow());
    }

    Ok(())
}
