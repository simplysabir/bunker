use anyhow::{anyhow, Result};
use chrono::Utc;
use colored::*;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(key: String, value: Option<String>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;
    
    // Load existing entry
    let mut entry = storage.load_entry(&key, &master_key)?;
    
    // Get new value
    let new_value = if let Some(v) = value {
        v
    } else {
        // Decrypt current value to show as default
        let current_decrypted = Crypto::decrypt(&entry.value, &master_key)?;
        let current_value = String::from_utf8(current_decrypted)
            .map_err(|e| anyhow!("Failed to decode current value: {}", e))?;
        
        println!("Current value: {}", utils::mask_password(&current_value, 3));
        utils::prompt_password(&format!("Enter new password for '{}'", key))?
    };
    
    // Encrypt new value
    let encrypted_value = Crypto::encrypt(new_value.as_bytes(), &master_key)?;
    
    // Update entry
    entry.value = encrypted_value;
    entry.updated_at = Utc::now();
    
    // Store updated entry
    storage.store_entry(&entry, &master_key)?;
    
    println!("{} Password '{}' updated successfully", "✓".green().bold(), key.cyan());
    
    Ok(())
}
