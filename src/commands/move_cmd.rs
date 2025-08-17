use anyhow::{anyhow, Result};
use colored::*;

use crate::storage::Storage;
use crate::utils;

pub async fn execute(from: String, to: String, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;
    
    // Load entry
    let mut entry = storage.load_entry(&from, &master_key)?;
    
    // Update key
    entry.key = to.clone();
    
    // Store with new key
    storage.store_entry(&entry, &master_key)?;
    
    // Delete old entry
    storage.delete_entry(&from)?;
    
    println!("{} Password moved from '{}' to '{}'", "✓".green().bold(), from.cyan(), to.cyan());
    
    Ok(())
}
