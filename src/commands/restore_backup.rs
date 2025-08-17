use anyhow::{Result, anyhow};
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::storage::Storage;
use crate::utils;

pub async fn execute(backup_path: PathBuf, vault_name: Option<String>) -> Result<()> {
    if !backup_path.exists() {
        return Err(anyhow!("Backup file not found: {}", backup_path.display()));
    }

    let vault_name = vault_name.unwrap_or_else(|| "restored".to_string());
    let storage = Storage::new(Some(vault_name.clone()))?;

    if storage.vault_exists() {
        if !utils::prompt_confirm(&format!(
            "Vault '{}' already exists. Overwrite?",
            vault_name
        ))? {
            return Ok(());
        }
        // Remove existing vault
        fs::remove_dir_all(storage.get_vault_path())?;
    }

    // Create vault directory
    fs::create_dir_all(storage.get_vault_path())?;

    // Extract backup
    let tar_gz = fs::File::open(&backup_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(storage.get_vault_path())?;

    println!(
        "{} Backup restored to vault '{}'",
        "✓".green().bold(),
        vault_name.cyan()
    );
    println!("You can now access your restored passwords");

    Ok(())
}
