use anyhow::{anyhow, Result};
use chrono::Utc;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::storage::Storage;

pub async fn execute(destination: Option<PathBuf>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Determine backup destination
    let backup_path = if let Some(dest) = destination {
        dest
    } else {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("bunker_backup_{}.tar.gz", timestamp);
        PathBuf::from(&backup_name)
    };
    
    // Create backup of entire vault directory
    let vault_path = storage.get_vault_path();
    
    // Create tar archive
    let tar_gz = fs::File::create(&backup_path)?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    
    tar.append_dir_all(".", vault_path)?;
    tar.finish()?;
    
    println!("{} Backup created: {}", "✓".green().bold(), backup_path.display().to_string().cyan());
    println!("This backup contains your entire vault including configuration and git history");
    
    Ok(())
}
