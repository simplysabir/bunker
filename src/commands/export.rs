use anyhow::{Result, anyhow};
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::types::ExportEntry;
use crate::utils;

pub async fn execute(
    format: String,
    output: Option<PathBuf>,
    include_metadata: bool,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;

    // Get all entries
    let entry_keys = storage.list_entries()?;
    let mut export_entries = Vec::new();

    for key in entry_keys {
        if let Ok(entry) = storage.load_entry(&key, &master_key) {
            if let Ok(decrypted) = Crypto::decrypt(&entry.value, &master_key) {
                if let Ok(value) = String::from_utf8(decrypted) {
                    let export_entry = ExportEntry {
                        key: entry.key,
                        value,
                        username: entry.metadata.username,
                        url: entry.metadata.url,
                        notes: if include_metadata {
                            entry.metadata.notes
                        } else {
                            None
                        },
                        tags: if include_metadata {
                            entry.metadata.tags
                        } else {
                            Vec::new()
                        },
                        created_at: entry.created_at,
                        updated_at: entry.updated_at,
                    };
                    export_entries.push(export_entry);
                }
            }
        }
    }

    // Store count before generating content
    let entry_count = export_entries.len();

    // Generate export content
    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&export_entries)?,
        "csv" => {
            let mut csv = String::from("key,value,username,url,notes,tags,created_at,updated_at\n");
            for entry in &export_entries {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\n",
                    entry.key,
                    entry.value,
                    entry.username.as_deref().unwrap_or_default(),
                    entry.url.as_deref().unwrap_or_default(),
                    entry.notes.as_deref().unwrap_or_default(),
                    entry.tags.join(";"),
                    entry.created_at,
                    entry.updated_at
                ));
            }
            csv
        }
        "pass" => {
            // Pass-style format
            let mut content = String::new();
            for entry in &export_entries {
                content.push_str(&format!("{}:\n{}\n", entry.key, entry.value));
                if let Some(username) = &entry.username {
                    content.push_str(&format!("username: {}\n", username));
                }
                if let Some(url) = &entry.url {
                    content.push_str(&format!("url: {}\n", url));
                }
                content.push('\n');
            }
            content
        }
        _ => {
            return Err(anyhow!(
                "Unsupported format: {}. Use json, csv, or pass",
                format
            ));
        }
    };

    // Write to file or stdout
    if let Some(path) = output {
        fs::write(&path, content)?;
        println!(
            "{} Exported {} entries to {}",
            "✓".green().bold(),
            entry_count,
            path.display().to_string().cyan()
        );
    } else {
        print!("{}", content);
    }

    Ok(())
}
