use anyhow::{anyhow, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::cli::Cli;
use crate::crypto::Crypto;
use crate::git::Git;
use crate::storage::Storage;
use crate::types::{Entry, EntryMetadata, EntryType};
use crate::utils;

pub async fn execute(
    key: String,
    value: Option<String>,
    note: bool,
    file: Option<PathBuf>,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;
    
    // Determine entry type and value
    let (entry_type, entry_value) = if let Some(file_path) = file {
        // Read file content
        let content = fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read file: {}", e))?;
        (EntryType::SecureFile, content)
    } else if note {
        // Get note content
        let content = value.unwrap_or_else(|| {
            println!("Enter note content (press Ctrl+D when done):");
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            buffer
        });
        (EntryType::Note, content)
    } else {
        // Get password
        let password = if let Some(v) = value {
            v
        } else {
            utils::prompt_password(&format!("Enter password for '{}'", key))?
        };
        (EntryType::Password, password)
    };
    
    // Create metadata
    let metadata = EntryMetadata {
        entry_type,
        tags: Vec::new(),
        notes: None,
        url: None,
        username: None,
        custom_fields: std::collections::HashMap::new(),
        expires_at: None,
        auto_type: None,
    };
    
    // Encrypt the value
    let encrypted_value = Crypto::encrypt(entry_value.as_bytes(), &master_key)?;
    
    // Create entry
    let entry = Entry {
        id: Uuid::new_v4(),
        key: key.clone(),
        value: encrypted_value,
        metadata,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        accessed_at: None,
    };
    
    // Store entry
    storage.store_entry(&entry, &master_key)?;
    
    // Commit if git enabled
    let config = storage.load_config()?;
    if Git::is_repo(storage.get_vault_path())? {
        Git::commit(storage.get_vault_path(), &format!("Add {}", key))?;
        
        if config.auto_sync && config.git_remote.is_some() {
            let _ = Git::push(storage.get_vault_path());
        }
    }
    
    Cli::print_entry_added(&key);
    
    Ok(())
}