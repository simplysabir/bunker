use anyhow::{anyhow, Result};
use colored::*;

use crate::storage::Storage;
use crate::utils;

pub async fn execute(vault: Option<String>, duration: Option<u64>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get master key (this will create a session if needed)
    let _master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;
    
    let duration_hours = duration.unwrap_or(24);
    
    println!("{} Vault unlocked for {} hours", "🔓".green().bold(), duration_hours);
    println!("Your passwords are now accessible without re-entering credentials");
    
    Ok(())
}
