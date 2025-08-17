use anyhow::{Result, anyhow};
use colored::*;

use crate::git::Git;
use crate::storage::Storage;

pub async fn execute(
    key: Option<String>,
    limit: Option<usize>,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    let vault_path = storage.get_vault_path();

    // Check if git is initialized
    if !Git::is_repo(vault_path)? {
        println!(
            "{}",
            "No version history available. Initialize git with 'bunker init' to track changes."
                .yellow()
        );
        return Ok(());
    }

    let history = if let Some(entry_key) = key {
        // Show history for specific entry
        let entry_path = format!(
            "store/{}.json",
            entry_key.replace('/', std::path::MAIN_SEPARATOR_STR)
        );
        Git::log_file(vault_path, &entry_path, limit)?
    } else {
        // Show general vault history
        Git::log(vault_path, limit)?
    };

    if history.is_empty() {
        println!("{}", "No history found".yellow());
        return Ok(());
    }

    println!("{} History:", "📜".green());
    for (i, commit) in history.iter().enumerate() {
        let prefix = if i == history.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!(
            "{} {} {}",
            prefix,
            commit.hash[..8].yellow(),
            commit.message
        );
        println!(
            "{}   {} by {} on {}",
            if i == history.len() - 1 {
                "    "
            } else {
                "│   "
            },
            "authored".dimmed(),
            commit.author.blue(),
            commit
                .timestamp
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .dimmed()
        );
        if i < history.len() - 1 {
            println!("│");
        }
    }

    Ok(())
}
