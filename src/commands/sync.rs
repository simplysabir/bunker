use anyhow::{anyhow, Result};
use colored::*;
use crate::cli::Cli;
use crate::git::Git;
use crate::storage::Storage;

pub async fn execute(message: Option<String>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    if !Git::is_repo(storage.get_vault_path())? {
        return Err(anyhow!("Git not initialized for this vault"));
    }
    
    // Check for changes
    let changes = Git::status(storage.get_vault_path())?;
    
    if changes.is_empty() {
        println!("No changes to sync");
        return Ok(());
    }
    
    // Show changes
    println!("Changes to sync:");
    for change in &changes {
        println!("  {}", change);
    }
    
    // Commit changes
    let commit_message = message.unwrap_or_else(|| {
        format!("Update vault ({})", chrono::Utc::now().format("%Y-%m-%d %H:%M"))
    });
    
    Git::commit(storage.get_vault_path(), &commit_message)?;
    
    // Push if remote configured
    let config = storage.load_config()?;
    if config.git_remote.is_some() {
        match Git::push(storage.get_vault_path()) {
            Ok(_) => Cli::print_sync_success(),
            Err(e) => {
                println!("{} Failed to push: {}", "⚠".yellow(), e);
                println!("Changes committed locally. Try 'git push' manually.");
            }
        }
    } else {
        println!("{} Changes committed locally", "✓".green().bold());
        println!("No remote configured. Add with: git remote add origin <url>");
    }
    
    Ok(())
}