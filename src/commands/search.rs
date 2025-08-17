use anyhow::{Result, anyhow};
use colored::*;
use skim::prelude::*;
use std::io::Cursor;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(query: Option<String>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault.clone())?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key for decryption
    let master_key = utils::get_master_key(vault.clone())?;

    if let Some(q) = query {
        // Search with provided query through decrypted content
        let results = storage.search_entries(&q, &master_key)?;

        if results.is_empty() {
            println!("{}", "No matches found".yellow());
        } else {
            println!(
                "{} Found {} matches:\n",
                "🔍".green(),
                results.len().to_string().bold()
            );
            for (entry_key, entry) in results {
                println!(
                    "  {} ({})",
                    entry_key.cyan(),
                    entry.metadata.entry_type.to_string().yellow()
                );

                // Show matching fields
                let mut matches = Vec::new();

                // Check username
                if let Some(username) = &entry.metadata.username {
                    if username.to_lowercase().contains(&q.to_lowercase()) {
                        matches.push(format!("username: {}", username));
                    }
                }

                // Check notes
                if let Some(notes) = &entry.metadata.notes {
                    if notes.to_lowercase().contains(&q.to_lowercase()) {
                        matches.push(format!(
                            "notes: {}",
                            notes.chars().take(50).collect::<String>()
                        ));
                    }
                }

                // Check custom fields
                for (field_name, field_value) in &entry.metadata.custom_fields {
                    if field_value.to_lowercase().contains(&q.to_lowercase()) {
                        matches.push(format!("{}: {}", field_name, field_value));
                    }
                }

                // Check URL
                if let Some(url) = &entry.metadata.url {
                    if url.to_lowercase().contains(&q.to_lowercase()) {
                        matches.push(format!("url: {}", url));
                    }
                }

                // Check tags
                for tag in &entry.metadata.tags {
                    if tag.to_lowercase().contains(&q.to_lowercase()) {
                        matches.push(format!("tag: {}", tag));
                    }
                }

                // Show what matched
                if !matches.is_empty() {
                    for m in matches {
                        println!("    {}", m.green());
                    }
                }
                println!();
            }
        }
    } else {
        // Interactive fuzzy search with skim - searches through decrypted content but shows clean interface
        let entries = storage.list_entries()?;

        if entries.is_empty() {
            println!("{}", "No passwords stored yet".yellow());
            return Ok(());
        }

        // Create searchable items with decrypted content for searching but clean display
        let mut search_items = Vec::new();

        for entry_key in &entries {
            if let Ok(entry) = storage.load_entry(entry_key, &master_key) {
                // Decrypt the password/value for searching (but don't show it)
                let decrypted_value = match Crypto::decrypt(&entry.value, &master_key) {
                    Ok(value) => String::from_utf8(value).unwrap_or_default(),
                    Err(_) => String::new(),
                };

                // Build clean display text (no passwords exposed)
                let mut display_text = format!("{} ({})", entry_key, entry.metadata.entry_type);

                if let Some(username) = &entry.metadata.username {
                    display_text.push_str(&format!(" | username: {}", username));
                }

                if let Some(notes) = &entry.metadata.notes {
                    display_text.push_str(&format!(" | notes: {}", notes));
                }

                if let Some(url) = &entry.metadata.url {
                    display_text.push_str(&format!(" | url: {}", url));
                }

                for tag in &entry.metadata.tags {
                    display_text.push_str(&format!(" | tag: {}", tag));
                }

                for (field_name, field_value) in &entry.metadata.custom_fields {
                    display_text.push_str(&format!(" | {}: {}", field_name, field_value));
                }

                // Store both clean display and searchable content (with decrypted password)
                let searchable_content = format!("{} | {}", display_text, decrypted_value);
                search_items.push((entry_key.clone(), display_text, searchable_content));
            }
        }

        // Use skim fuzzy finder with the searchable content (includes decrypted passwords for searching)
        let options = SkimOptionsBuilder::default()
            .prompt(Some("Search (searches through all content): "))
            .preview(Some(""))
            .build()
            .unwrap();

        // Use the searchable content (includes decrypted passwords) for the fuzzy finder
        let input = search_items
            .iter()
            .map(|(_, _, searchable_content)| searchable_content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(input));

        let selected = Skim::run_with(&options, Some(items))
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        if !selected.is_empty() {
            let selected_text = selected[0].output().to_string();

            // Find the corresponding entry key by matching the searchable content
            let entry_key = search_items
                .iter()
                .find(|(_, _, searchable_content)| searchable_content == &selected_text)
                .map(|(key, _, _)| key.clone())
                .ok_or_else(|| anyhow!("Failed to find selected entry"))?;

            // Ask what to do with the selected entry
            println!("\nSelected: {}", entry_key.cyan().bold());
            println!("\nWhat would you like to do?");
            println!("  {} Copy password", "1.".blue());
            println!("  {} Show password", "2.".blue());
            println!("  {} Edit entry", "3.".blue());
            println!("  {} Cancel", "4.".blue());

            let action = utils::prompt_input("Choice (1-4)")?;

            match action.as_str() {
                "1" => super::copy::execute(entry_key.clone(), false, 45, vault.clone()).await?,
                "2" => super::get::execute(entry_key.clone(), false, vault.clone()).await?,
                "3" => super::edit::execute(entry_key, None, vault).await?,
                _ => println!("Cancelled"),
            }
        }
    }

    Ok(())
}
