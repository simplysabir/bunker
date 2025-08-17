use anyhow::{anyhow, Result};
use colored::*;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(key: String, var_name: Option<String>, vault: Option<String>) -> Result<()> {
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
    let password = String::from_utf8(decrypted)
        .map_err(|e| anyhow!("Failed to decode value: {}", e))?;
    
    // Determine variable name
    let env_var = var_name.unwrap_or_else(|| {
        key.to_uppercase().replace('/', "_").replace('-', "_")
    });
    
    // Output export statement
    println!("export {}='{}'", env_var, password);
    
    // Provide usage hint
    eprintln!("{} Use: {}", 
        "💡".yellow(), 
        format!("eval \"$(bunker env {})\"", key).cyan()
    );
    
    Ok(())
}
