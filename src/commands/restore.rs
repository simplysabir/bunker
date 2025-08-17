use anyhow::{anyhow, Result};
use colored::*;

use crate::git::Git;
use crate::storage::Storage;

pub async fn execute(commit_hash: String, key: Option<String>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    let vault_path = storage.get_vault_path();
    
    // Check if git is initialized
    if !Git::is_repo(vault_path)? {
        return Err(anyhow!("Git not initialized for this vault"));
    }
    
    if let Some(entry_key) = key {
        // Restore specific entry
        let entry_path = format!("store/{}.json", entry_key.replace('/', std::path::MAIN_SEPARATOR_STR));
        Git::restore_file(vault_path, &commit_hash, &entry_path)?;
        println!("{} Restored '{}' from commit {}", 
            "✓".green().bold(), 
            entry_key.cyan(),
            commit_hash[..8].yellow()
        );
    } else {
        // Restore entire vault
        if !crate::utils::prompt_confirm(&format!(
            "This will restore the entire vault to commit {}. Continue?", 
            &commit_hash[..8]
        ))? {
            return Ok(());
        }
        
        Git::restore_commit(vault_path, &commit_hash)?;
        println!("{} Restored vault to commit {}", 
            "✓".green().bold(), 
            commit_hash[..8].yellow()
        );
    }
    
    Ok(())
}
