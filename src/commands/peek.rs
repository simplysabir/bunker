use crate::cli::Cli;
use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;
use anyhow::{Result, anyhow};
use colored::*;

pub async fn execute(key: String, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;

    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }

    // Get master key
    let master_key = utils::get_master_key(Some(storage.get_vault_name().to_string()))?;

    // Load entry
    let entry = storage.load_entry(&key, &master_key)?;

    // Decrypt the value
    let decrypted = Crypto::decrypt(&entry.value, &master_key)?;
    let password =
        String::from_utf8(decrypted).map_err(|e| anyhow!("Failed to decode value: {}", e))?;

    // Mask the password
    let masked = utils::mask_password(&password, 2);

    Cli::print_masked_password(&key, &masked);

    // Clear screen after a short delay
    println!("\n{}", "Press Enter to clear...".dimmed());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    utils::clear_screen();

    Ok(())
}
