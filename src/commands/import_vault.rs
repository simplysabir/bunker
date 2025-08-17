use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use colored::*;

use crate::cli::Cli;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(file: PathBuf, password: String, name: String) -> Result<()> {
    if !file.exists() {
        return Err(anyhow!("Import file not found: {}", file.display()));
    }
    
    // Read import file
    let import_data = fs::read(&file)
        .map_err(|e| anyhow!("Failed to read import file: {}", e))?;
    
    // Check if vault already exists
    let storage = Storage::new(Some(name.clone()))?;
    if storage.vault_exists() {
        if !utils::prompt_confirm(&format!("Vault '{}' already exists. Overwrite?", name))? {
            return Ok(());
        }
        // Remove existing vault
        fs::remove_dir_all(storage.get_vault_path())?;
    }
    
    println!("{} Importing vault from {}...", "🔄".blue(), file.display().to_string().cyan());
    
    // Import vault
    Storage::import_vault(&import_data, &password, &name)?;
    
    println!("{} Vault '{}' imported successfully!", "✓".green().bold(), name.cyan());
    println!("🔐 All your passwords are now available on this device");
    
    // Automatically switch to the imported vault
    let mut config = crate::config::Config::load()?;
    config.default_vault = name.clone();
    config.save()?;
    
    println!("\n{} Vault '{}' is now your default vault", "🏠".green(), name.cyan());
    
    // Show quick access commands
    println!("\n{} Quick actions:", "💡".yellow().bold());
    println!("  {} List all passwords: {}", "•".blue(), "bunker list".white().bold());
    println!("  {} Get a password: {}", "•".blue(), "bunker get <name>".white().bold());
    println!("  {} Add new password: {}", "•".blue(), "bunker add <name>".white().bold());
    println!("  {} Search passwords: {}", "•".blue(), "bunker search".white().bold());
    
    // Show unlock info
    println!("\n{} Your vault is now unlocked for 24 hours", "⏰".yellow());
    println!("Run 'bunker unlock' to extend or 'bunker lock' to secure immediately");
    
    Ok(())
}