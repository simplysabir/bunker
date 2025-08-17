use anyhow::{Result, anyhow};
use chrono::Utc;
use colored::*;
use std::collections::HashMap;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::types::{EntryMetadata, EntryType};
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

    println!("{} Editing entry '{}'", "✏️".blue(), key.cyan().bold());
    println!(
        "Current type: {}",
        entry.metadata.entry_type.to_string().yellow()
    );

    // Show current value (masked for security)
    let current_decrypted = Crypto::decrypt(&entry.value, &master_key)?;
    let current_value = String::from_utf8(current_decrypted)
        .map_err(|e| anyhow!("Failed to decode current value: {}", e))?;
    println!("Current value: {}", utils::mask_password(&current_value, 3));

    // Ask what to edit
    println!("\nWhat would you like to edit?");
    println!("  {} Password/Value", "1.".blue());
    println!("  {} Entry Type", "2.".blue());
    println!("  {} Username", "3.".blue());
    println!("  {} Notes", "4.".blue());
    println!("  {} URL", "5.".blue());
    println!("  {} Tags", "6.".blue());
    println!("  {} Custom Fields", "7.".blue());
    println!("  {} Cancel", "8.".blue());

    let choice = utils::prompt_input("Choice (1-8): ")?;

    match choice.as_str() {
        "1" => {
            // Edit password/value
            let new_value = if let Some(v) = value {
                v
            } else {
                utils::prompt_password(&format!("Enter new value for '{}': ", key))?
            };

            let encrypted_value = Crypto::encrypt(new_value.as_bytes(), &master_key)?;
            entry.value = encrypted_value;
            println!("{} Value updated", "✓".green());
        }

        "2" => {
            // Edit entry type
            println!(
                "Current type: {}",
                entry.metadata.entry_type.to_string().cyan()
            );
            println!("Available types:");
            println!("  {} Password", "1.".blue());
            println!("  {} Note", "2.".blue());
            println!("  {} Card", "3.".blue());
            println!("  {} Identity", "4.".blue());
            println!("  {} SecureFile", "5.".blue());
            println!("  {} ApiKey", "6.".blue());
            println!("  {} SshKey", "7.".blue());
            println!("  {} Database", "8.".blue());
            println!("  {} Custom", "9.".blue());

            let type_choice = utils::prompt_input("Choose new type (1-9): ")?;
            let new_type = match type_choice.as_str() {
                "1" => EntryType::Password,
                "2" => EntryType::Note,
                "3" => EntryType::Card,
                "4" => EntryType::Identity,
                "5" => EntryType::SecureFile,
                "6" => EntryType::ApiKey,
                "7" => EntryType::SshKey,
                "8" => EntryType::Database,
                "9" => {
                    let custom_type = utils::prompt_input("Enter custom type: ")?;
                    EntryType::Custom(custom_type)
                }
                _ => {
                    println!("{} Invalid choice, keeping current type", "⚠️".yellow());
                    entry.metadata.entry_type.clone()
                }
            };

            if new_type != entry.metadata.entry_type {
                entry.metadata.entry_type = new_type;
                println!("{} Entry type updated", "✓".green());
            }
        }

        "3" => {
            // Edit username
            let current_username = entry.metadata.username.as_deref().unwrap_or("None");
            println!("Current username: {}", current_username.cyan());
            let new_username = utils::prompt_input("Enter new username (empty to remove): ")?;
            if new_username.trim().is_empty() {
                entry.metadata.username = None;
                println!("{} Username removed", "✓".green());
            } else {
                entry.metadata.username = Some(new_username);
                println!("{} Username updated", "✓".green());
            }
        }

        "4" => {
            // Edit notes
            let current_notes = entry.metadata.notes.as_deref().unwrap_or("None");
            println!("Current notes: {}", current_notes.cyan());
            let new_notes = utils::prompt_input("Enter new notes (empty to remove): ")?;
            if new_notes.trim().is_empty() {
                entry.metadata.notes = None;
                println!("{} Notes removed", "✓".green());
            } else {
                entry.metadata.notes = Some(new_notes);
                println!("{} Notes updated", "✓".green());
            }
        }

        "5" => {
            // Edit URL
            let current_url = entry.metadata.url.as_deref().unwrap_or("None");
            println!("Current URL: {}", current_url.cyan());
            let new_url = utils::prompt_input("Enter new URL (empty to remove): ")?;
            if new_url.trim().is_empty() {
                entry.metadata.url = None;
                println!("{} URL removed", "✓".green());
            } else {
                entry.metadata.url = Some(new_url);
                println!("{} URL updated", "✓".green());
            }
        }

        "6" => {
            // Edit tags
            let current_tags = if entry.metadata.tags.is_empty() {
                "None".to_string()
            } else {
                entry.metadata.tags.join(", ")
            };
            println!("Current tags: {}", current_tags.cyan());
            let new_tags_input =
                utils::prompt_input("Enter new tags (comma-separated, empty to remove): ")?;
            if new_tags_input.trim().is_empty() {
                entry.metadata.tags.clear();
                println!("{} Tags removed", "✓".green());
            } else {
                let new_tags: Vec<String> = new_tags_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                entry.metadata.tags = new_tags;
                println!("{} Tags updated", "✓".green());
            }
        }

        "7" => {
            // Edit custom fields
            if entry.metadata.custom_fields.is_empty() {
                println!("No custom fields currently set");
            } else {
                println!("Current custom fields:");
                for (field_name, field_value) in &entry.metadata.custom_fields {
                    println!("  {}: {}", field_name.cyan(), field_value.green());
                }
            }

            let action = utils::prompt_input(
                "Action: 'add' to add field, 'remove' to remove field, 'cancel' to skip: ",
            )?;
            match action.to_lowercase().as_str() {
                "add" => {
                    let field_name = utils::prompt_input("Enter field name: ")?;
                    let field_value = utils::prompt_input("Enter field value: ")?;
                    entry.metadata.custom_fields.insert(field_name, field_value);
                    println!("{} Custom field added", "✓".green());
                }
                "remove" => {
                    let field_name = utils::prompt_input("Enter field name to remove: ")?;
                    if entry.metadata.custom_fields.remove(&field_name).is_some() {
                        println!("{} Custom field removed", "✓".green());
                    } else {
                        println!("{} Field not found", "⚠️".yellow());
                    }
                }
                _ => println!("Skipping custom fields"),
            }
        }

        "8" => {
            println!("{} Edit cancelled", "⚠️".yellow());
            return Ok(());
        }

        _ => {
            println!("{} Invalid choice", "⚠️".red());
            return Ok(());
        }
    }

    // Update timestamp
    entry.updated_at = Utc::now();

    // Store updated entry
    storage.store_entry(&entry, &master_key)?;

    println!(
        "{} Entry '{}' updated successfully",
        "✓".green().bold(),
        key.cyan()
    );

    Ok(())
}
