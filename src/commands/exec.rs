use anyhow::{Result, anyhow};
use std::process::Command;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::utils;

pub async fn execute(
    command: Vec<String>,
    key: String,
    env: Option<String>,
    vault: Option<String>,
) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("No command specified"));
    }

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

    // Prepare command
    let program = &command[0];
    let args: Vec<String> = if env.is_some() {
        // Use as environment variable
        command[1..].to_vec()
    } else {
        // Replace placeholder or append
        command[1..]
            .iter()
            .map(|arg| {
                if arg == "{}" || arg == "$PASSWORD" {
                    password.clone()
                } else {
                    arg.clone()
                }
            })
            .collect()
    };

    // Execute command
    let mut cmd = Command::new(program);
    cmd.args(&args);

    if let Some(env_var) = env {
        cmd.env(env_var, &password);
    }

    let status = cmd
        .status()
        .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
