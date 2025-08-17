use anyhow::{Result, anyhow};

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::types::EntryType;
use crate::utils;
use colored::*;

pub async fn execute(key: String, quiet: bool, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;

    // Load entry
    let entry = storage.load_entry(&key, &master_key)?;

    // Decrypt the actual value
    let decrypted = Crypto::decrypt(&entry.value, &master_key)?;
    let value =
        String::from_utf8(decrypted).map_err(|e| anyhow!("Failed to decode value: {}", e))?;

    if quiet {
        // Just print the value
        print!("{}", value);
    } else {
        // Print with metadata
        println!("{}: {}", key.cyan().bold(), value);

        if !matches!(entry.metadata.entry_type, EntryType::Password) {
            println!("Type: {:?}", entry.metadata.entry_type);
        }

        if let Some(url) = &entry.metadata.url {
            println!("URL: {}", url);
        }

        if let Some(username) = &entry.metadata.username {
            println!("Username: {}", username);
        }

        if let Some(notes) = &entry.metadata.notes {
            println!("Notes: {}", notes);
        }

        if !entry.metadata.tags.is_empty() {
            println!("Tags: {}", entry.metadata.tags.join(", "));
        }
    }

    Ok(())
}
