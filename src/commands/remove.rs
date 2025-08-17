use anyhow::{Result, anyhow};

use crate::cli::Cli;
use crate::git::Git;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(key: String, force: bool, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Confirm deletion
    if !force {
        if !utils::prompt_confirm(&format!("Remove password '{}'?", key))? {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Delete entry
    storage.delete_entry(&key)?;

    // Commit if git enabled
    if Git::is_repo(storage.get_vault_path())? {
        Git::commit(storage.get_vault_path(), &format!("Remove {}", key))?;

        let config = storage.load_config()?;
        if config.auto_sync && config.git_remote.is_some() {
            let _ = Git::push(storage.get_vault_path());
        }
    }

    Cli::print_entry_removed(&key);

    Ok(())
}
