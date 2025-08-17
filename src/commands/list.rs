use anyhow::{Result, anyhow};
use colored::*;

use crate::storage::Storage;
use crate::utils;

pub async fn execute(path: Option<String>, flat: bool, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // List all entries
    let entries = storage.list_entries()?;

    if entries.is_empty() {
        println!("{}", "No passwords stored yet".yellow());
        println!(
            "Add your first password with: {}",
            "bunker add <key>".white().bold()
        );
        return Ok(());
    }

    // Filter by path if provided
    let filtered: Vec<String> = if let Some(p) = path {
        entries.into_iter().filter(|e| e.starts_with(&p)).collect()
    } else {
        entries
    };

    if filtered.is_empty() {
        println!("{}", "No passwords found in this path".yellow());
        return Ok(());
    }

    println!(
        "{} {} passwords:\n",
        "🔐".green(),
        filtered.len().to_string().bold()
    );

    if flat {
        // Flat list
        for entry in filtered {
            println!("  {}", entry.cyan());
        }
    } else {
        // Tree view
        let tree = utils::format_tree(&filtered, "");
        print!("{}", tree);
    }

    Ok(())
}
