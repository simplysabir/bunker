use anyhow::{anyhow, Result};

use crate::cli::Cli;
use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(
    key: String,
    persist: bool,
    timeout: u64,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;
    
    // Load entry
    let entry = storage.load_entry(&key, &master_key)?;
    
    // Decrypt the value
    let decrypted = Crypto::decrypt(&entry.value, &master_key)?;
    let value = String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Failed to decode value: {}", e))?;
    
    // Copy to clipboard
    let actual_timeout = if persist { 0 } else { timeout };
    utils::copy_to_clipboard(&value, actual_timeout)?;
    
    Cli::print_entry_copied(&key, actual_timeout);
    
    Ok(())
}