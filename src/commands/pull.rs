use anyhow::{Result, anyhow};
use colored::*;

use crate::git::Git;
use crate::storage::Storage;

pub async fn execute(vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    let vault_path = storage.get_vault_path();

    // Check if git is initialized
    if !Git::is_repo(vault_path)? {
        return Err(anyhow!("Git not initialized for this vault"));
    }

    // Check if remote is configured
    let config = storage.load_config()?;
    if config.git_remote.is_none() {
        return Err(anyhow!(
            "No git remote configured. Add one with 'bunker vault add-remote <url>'"
        ));
    }

    // Pull changes
    let result = Git::pull(vault_path)?;

    if result.is_empty() {
        println!("{} Already up to date", "✓".green().bold());
    } else {
        println!("{} Pulled changes from remote:", "✓".green().bold());
        for commit in result {
            println!("  {} {}", commit.hash[..8].yellow(), commit.message);
        }
    }

    Ok(())
}
