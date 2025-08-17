use anyhow::{Result, anyhow};
use chrono::Utc;
use colored::*;
use uuid::Uuid;

use crate::cli::Cli;
use crate::config::Config;
use crate::crypto::Crypto;
use crate::git::Git;
use crate::storage::Storage;
use crate::types::{EncryptionConfig, VaultConfig};
use crate::utils;

pub async fn execute(name: String, non_interactive: bool, vault: Option<String>) -> Result<()> {
    let vault_name = vault.unwrap_or(name.clone());
    let storage = Storage::new(Some(vault_name.clone()))?;

    if storage.vault_exists() {
        return Err(anyhow!("Vault '{}' already exists", vault_name));
    }

    if !non_interactive {
        Cli::print_banner();
        Cli::print_welcome();
    }

    // Get master password
    let password = if non_interactive {
        utils::prompt_password("Enter master password")?
    } else {
        println!("\n{}", "Setting up your vault...".cyan());
        println!("Choose a strong master password. This will encrypt all your passwords.");
        println!(
            "{}",
            "You'll need this password to access your vault.".yellow()
        );
        utils::prompt_password_confirm("\nMaster password")?
    };

    // Create vault configuration
    let config = VaultConfig {
        id: Uuid::new_v4(),
        name: vault_name.clone(),
        created_at: Utc::now(),
        last_modified: Utc::now(),
        encryption: EncryptionConfig::default(),
        git_remote: None,
        auto_sync: true,
        auto_lock_minutes: Some(15),
    };

    // Initialize vault
    storage.init_vault(config.clone())?;

    // Set up permanent master key storage
    let master_key = Crypto::derive_key(&password, config.id.as_bytes())?;
    storage.store_master_key_permanently(&master_key)?;

    // Initialize git repository
    if !non_interactive {
        if utils::prompt_confirm("Initialize git repository for version control?")? {
            Git::init(storage.get_vault_path())?;
            Git::commit(storage.get_vault_path(), "Initial vault setup")?;

            if let Some(remote) = utils::prompt_input_optional("Git remote URL (optional)")? {
                Git::add_remote(storage.get_vault_path(), &remote)?;

                let mut updated_config = config;
                updated_config.git_remote = Some(remote);
                storage.save_config(&updated_config)?;
            }
        }
    }

    // Update global config
    let mut global_config = Config::load()?;
    if global_config.default_vault == "default" && vault_name != "default" {
        global_config.default_vault = vault_name.clone();
        global_config.save()?;
    }

    if !non_interactive {
        Cli::print_init_success(&vault_name);
    } else {
        println!("Vault '{}' initialized", vault_name);
    }

    Ok(())
}
