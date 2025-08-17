use anyhow::{Result, anyhow};
use colored::*;

use crate::cli::VaultAction;
use crate::config::Config;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(action: VaultAction) -> Result<()> {
    match action {
        VaultAction::Create { name } => create_vault(name).await,
        VaultAction::Use { name } => use_vault(name).await,
        VaultAction::List => list_vaults().await,
        VaultAction::Delete { name, force } => delete_vault(name, force).await,
        VaultAction::Export { password, output } => {
            crate::commands::export_vault::execute(password, output, None).await
        }
        VaultAction::Import {
            file,
            password,
            name,
        } => crate::commands::import_vault::execute(file, password, name).await,
    }
}

async fn create_vault(name: String) -> Result<()> {
    // Check if vault exists
    let storage = Storage::new(Some(name.clone()))?;
    if storage.vault_exists() {
        return Err(anyhow!("Vault '{}' already exists", name));
    }

    // Create the vault
    super::init::execute(name.clone(), false, Some(name.clone())).await?;

    Ok(())
}

async fn use_vault(name: String) -> Result<()> {
    // Check if vault exists
    let storage = Storage::new(Some(name.clone()))?;
    if !storage.vault_exists() {
        return Err(anyhow!("Vault '{}' does not exist", name));
    }

    // Update default vault in config
    let mut config = Config::load()?;
    config.default_vault = name.clone();
    config.save()?;

    println!("{} Switched to vault '{}'", "✓".green().bold(), name.cyan());

    Ok(())
}

async fn list_vaults() -> Result<()> {
    let vaults = Storage::list_vaults()?;

    if vaults.is_empty() {
        println!("{}", "No vaults found".yellow());
        println!(
            "Create your first vault with: {}",
            "bunker init".white().bold()
        );
        return Ok(());
    }

    let config = Config::load()?;

    println!(
        "{} {} vaults:\n",
        "🗄️".green(),
        vaults.len().to_string().bold()
    );

    for vault in vaults {
        let marker = if vault == config.default_vault {
            " (default)".green().to_string()
        } else {
            String::new()
        };

        println!("  {}{}", vault.cyan(), marker);
    }

    Ok(())
}

async fn delete_vault(name: String, force: bool) -> Result<()> {
    // Prevent deleting default vault
    let config = Config::load()?;
    if name == "default" || name == config.default_vault {
        return Err(anyhow!(
            "Cannot delete the default vault. Switch to another vault first."
        ));
    }

    // Check if vault exists
    let storage = Storage::new(Some(name.clone()))?;
    if !storage.vault_exists() {
        return Err(anyhow!("Vault '{}' does not exist", name));
    }

    // Confirm deletion
    if !force {
        println!(
            "{}",
            "⚠️  This will permanently delete all passwords in this vault!"
                .red()
                .bold()
        );
        if !utils::prompt_confirm(&format!("Delete vault '{}'?", name))? {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Delete vault directory
    std::fs::remove_dir_all(storage.get_vault_path())?;

    println!("{} Vault '{}' deleted", "✓".green().bold(), name.cyan());

    Ok(())
}
