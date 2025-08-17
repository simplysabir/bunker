use anyhow::{Result, anyhow};
use chrono::Utc;
use colored::*;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::types::{Entry, EntryMetadata, EntryType, ExportEntry};
use crate::utils;

pub async fn execute(
    file: PathBuf,
    format: String,
    overwrite: bool,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;

    // Read file
    let content = fs::read_to_string(&file)?;

    // Parse based on format
    let import_entries: Vec<ExportEntry> = match format.as_str() {
        "json" => serde_json::from_str(&content)?,
        "csv" => {
            let mut entries = Vec::new();
            let mut lines = content.lines();
            let _header = lines.next(); // Skip header

            for line in lines {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let entry = ExportEntry {
                        key: parts[0].to_string(),
                        value: parts[1].to_string(),
                        username: if parts.len() > 2 && !parts[2].is_empty() {
                            Some(parts[2].to_string())
                        } else {
                            None
                        },
                        url: if parts.len() > 3 && !parts[3].is_empty() {
                            Some(parts[3].to_string())
                        } else {
                            None
                        },
                        notes: if parts.len() > 4 && !parts[4].is_empty() {
                            Some(parts[4].to_string())
                        } else {
                            None
                        },
                        tags: if parts.len() > 5 && !parts[5].is_empty() {
                            parts[5].split(';').map(|s| s.to_string()).collect()
                        } else {
                            Vec::new()
                        },
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    entries.push(entry);
                }
            }
            entries
        }
        _ => return Err(anyhow!("Unsupported format: {}. Use json or csv", format)),
    };

    let mut imported = 0;
    let mut skipped = 0;

    for import_entry in import_entries {
        // Check if entry exists
        if storage.load_entry(&import_entry.key, &master_key).is_ok() && !overwrite {
            skipped += 1;
            continue;
        }

        // Create metadata
        let metadata = EntryMetadata {
            entry_type: EntryType::Password,
            tags: import_entry.tags,
            notes: import_entry.notes,
            url: import_entry.url,
            username: import_entry.username,
            custom_fields: std::collections::HashMap::new(),
            expires_at: None,
            auto_type: None,
        };

        // Encrypt value
        let encrypted_value = Crypto::encrypt(import_entry.value.as_bytes(), &master_key)?;

        // Create entry
        let entry = Entry {
            id: Uuid::new_v4(),
            key: import_entry.key,
            value: encrypted_value,
            metadata,
            created_at: import_entry.created_at,
            updated_at: Utc::now(),
            accessed_at: None,
        };

        // Store entry
        storage.store_entry(&entry, &master_key)?;
        imported += 1;
    }

    println!(
        "{} Imported {} entries",
        "✓".green().bold(),
        imported.to_string().cyan()
    );
    if skipped > 0 {
        println!(
            "{} Skipped {} existing entries (use --overwrite to replace)",
            "⚠".yellow(),
            skipped.to_string().yellow()
        );
    }

    Ok(())
}
