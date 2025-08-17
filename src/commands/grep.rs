use anyhow::{Result, anyhow};
use colored::*;
use regex::Regex;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(pattern: String, case_insensitive: bool, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;

    // Create regex
    let regex = if case_insensitive {
        Regex::new(&format!("(?i){}", pattern))?
    } else {
        Regex::new(&pattern)?
    };

    // Get all entries
    let entries = storage.list_entries()?;
    let mut matches = Vec::new();

    for entry_key in entries {
        // Load and decrypt entry
        if let Ok(entry) = storage.load_entry(&entry_key, &master_key) {
            if let Ok(decrypted) = Crypto::decrypt(&entry.value, &master_key) {
                if let Ok(value) = String::from_utf8(decrypted) {
                    // Search in key, value, and metadata
                    let mut match_contexts = Vec::new();

                    // Check key
                    if regex.is_match(&entry_key) {
                        match_contexts.push(("key", entry_key.clone()));
                    }

                    // Check value (masked)
                    if regex.is_match(&value) {
                        match_contexts.push(("value", utils::mask_password(&value, 3)));
                    }

                    // Check metadata
                    if let Some(username) = &entry.metadata.username {
                        if regex.is_match(username) {
                            match_contexts.push(("username", username.clone()));
                        }
                    }

                    if let Some(url) = &entry.metadata.url {
                        if regex.is_match(url) {
                            match_contexts.push(("url", url.clone()));
                        }
                    }

                    if let Some(notes) = &entry.metadata.notes {
                        if regex.is_match(notes) {
                            match_contexts.push(("notes", notes.clone()));
                        }
                    }

                    if !match_contexts.is_empty() {
                        matches.push((entry_key, match_contexts));
                    }
                }
            }
        }
    }

    if matches.is_empty() {
        println!(
            "{} No matches found for pattern '{}'",
            "🔍".yellow(),
            pattern
        );
    } else {
        println!(
            "{} Found {} matches for pattern '{}':\n",
            "🔍".green(),
            matches.len().to_string().bold(),
            pattern.cyan()
        );

        for (key, contexts) in matches {
            println!("{}", key.blue().bold());
            for (field, value) in contexts {
                println!("  {}: {}", field.yellow(), value);
            }
            println!();
        }
    }

    Ok(())
}
