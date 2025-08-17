use anyhow::{Result, anyhow};
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(
    password: String,
    output: Option<PathBuf>,
    vault: Option<String>,
) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Export vault with the provided password
    let exported_data = storage.export_vault(&password)?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let vault_name = storage.get_vault_name();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        PathBuf::from(format!("{}_{}.bunker", vault_name, timestamp))
    });

    // Write to file
    fs::write(&output_path, exported_data)?;

    println!("{} Vault exported successfully!", "✓".green().bold());
    println!(
        "📦 Export file: {}",
        output_path.display().to_string().cyan()
    );
    println!("🔐 Protected with your export password");

    // Show import instructions
    println!("\n{} To import on another device:", "💡".yellow().bold());
    println!("  1. Copy the export file to your new device");
    println!(
        "  2. Run: {}",
        format!(
            "bunker vault import {} <password> <vault-name>",
            output_path.file_name().unwrap().to_string_lossy()
        )
        .white()
        .bold()
    );
    println!("  3. Your passwords will be available immediately!");

    // Offer to generate import command
    if utils::prompt_confirm("Generate import command for easy copy-paste?")? {
        let import_cmd = format!(
            "bunker vault import {} {} {}",
            output_path.file_name().unwrap().to_string_lossy(),
            password,
            storage.get_vault_name()
        );
        println!("\n{} Import command (copy this):", "📋".blue().bold());
        println!("{}", import_cmd.white().bold().on_black());

        // Copy to clipboard if available
        if let Ok(_) = utils::copy_to_clipboard(&import_cmd, 0) {
            println!("{} Command copied to clipboard!", "✓".green());
        }
    }

    Ok(())
}
