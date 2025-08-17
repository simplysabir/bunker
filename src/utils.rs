use anyhow::{Result, anyhow};
use clipboard::{ClipboardContext, ClipboardProvider};
use colored::*;
use dialoguer::{Confirm, Input, Password};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::crypto::Crypto;
use crate::storage::Storage;
use crate::types::MasterKey;

/// Format error for display
pub fn format_error(err: &anyhow::Error) -> String {
    format!("{} {}", "✗".red().bold(), err.to_string().red())
}

/// Format success message
pub fn format_success(msg: &str) -> String {
    format!("{} {}", "✓".green().bold(), msg.green())
}

/// Format warning message
pub fn format_warning(msg: &str) -> String {
    format!("{} {}", "⚠".yellow().bold(), msg.yellow())
}

/// Format info message
pub fn format_info(msg: &str) -> String {
    format!("{} {}", "→".blue().bold(), msg)
}

/// Prompt for password
pub fn prompt_password(prompt: &str) -> Result<String> {
    let password = Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| anyhow!("Failed to read password: {}", e))?;

    if password.is_empty() {
        return Err(anyhow!("Password cannot be empty"));
    }

    Ok(password)
}

/// Prompt for password with confirmation
pub fn prompt_password_confirm(prompt: &str) -> Result<String> {
    let password = prompt_password(prompt)?;
    let confirm = prompt_password("Confirm password")?;

    if password != confirm {
        return Err(anyhow!("Passwords do not match"));
    }

    Ok(password)
}

/// Prompt for input
pub fn prompt_input(prompt: &str) -> Result<String> {
    let input = Input::<String>::new()
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;

    Ok(input)
}

/// Prompt for optional input
pub fn prompt_input_optional(prompt: &str) -> Result<Option<String>> {
    let input = Input::<String>::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompt for confirmation
pub fn prompt_confirm(prompt: &str) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| anyhow!("Failed to read confirmation: {}", e))?;

    Ok(confirmed)
}

/// Copy to clipboard
pub fn copy_to_clipboard(text: &str, timeout_seconds: u64) -> Result<()> {
    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| anyhow!("Failed to access clipboard: {}", e))?;

    ctx.set_contents(text.to_string())
        .map_err(|e| anyhow!("Failed to copy to clipboard: {}", e))?;

    if timeout_seconds > 0 {
        let clear_text = text.to_string();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(timeout_seconds));
            let mut ctx: ClipboardContext = match ClipboardProvider::new() {
                Ok(ctx) => ctx,
                Err(_) => return,
            };
            if let Ok(current) = ctx.get_contents() {
                // Only clear if clipboard still contains our text
                if current == clear_text {
                    let _ = ctx.set_contents(String::new());
                }
            }
        });
    }

    Ok(())
}

/// Mask a password for display
pub fn mask_password(password: &str, show_chars: usize) -> String {
    if password.len() <= show_chars * 2 {
        // Too short to mask meaningfully
        "*".repeat(password.len())
    } else {
        let start = &password[..show_chars];
        let end = &password[password.len() - show_chars..];
        format!(
            "{}{}{}",
            start,
            "*".repeat(password.len() - show_chars * 2),
            end
        )
    }
}

/// Format tree structure
pub fn format_tree(entries: &[String], prefix: &str) -> String {
    let mut tree = String::new();
    let mut path_tree: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    // Build tree structure
    for entry in entries {
        let parts: Vec<&str> = entry.split('/').collect();
        if parts.len() == 1 {
            path_tree
                .entry(String::new())
                .or_insert_with(Vec::new)
                .push(entry.clone());
        } else {
            let dir = parts[0].to_string();
            let rest = parts[1..].join("/");
            path_tree.entry(dir).or_insert_with(Vec::new).push(rest);
        }
    }

    // Render tree
    let mut items: Vec<_> = path_tree.iter().collect();
    items.sort_by_key(|&(k, _)| k);

    for (i, (dir, children)) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let connector = if is_last { "└──" } else { "├──" };

        if dir.is_empty() {
            // Root level entries
            for child in children.iter() {
                tree.push_str(&format!("{}{} {}\n", prefix, connector, child.cyan()));
            }
        } else {
            // Directory
            tree.push_str(&format!("{}{} {}/\n", prefix, connector, dir.blue().bold()));

            // Recursively format children
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            let child_tree = format_tree(children, &child_prefix);
            tree.push_str(&child_tree);
        }
    }

    tree
}

/// Get master key (from permanent storage) - Passwordless after setup
pub fn get_master_key(vault_name: Option<String>) -> Result<MasterKey> {
    let storage = Storage::new(vault_name)?;

    // Try to load from permanent storage first
    if let Ok(master_key) = storage.load_master_key_permanently() {
        return Ok(master_key);
    }

    // No permanent storage found - this should only happen on first use after vault creation
    // or if permanent storage was corrupted
    println!("{}", "🔐 Setting up passwordless access...".cyan());
    let password = prompt_password("Enter master password")?;

    // Derive key with vault-specific salt
    let config = storage.load_config()?;
    let salt = config.id.as_bytes();
    let master_key = Crypto::derive_key(&password, salt)?;

    // Store master key permanently for future use
    storage.store_master_key_permanently(&master_key)?;

    println!(
        "{}",
        "✓ Passwordless access configured. You'll never need to enter your password again!".green()
    );

    Ok(master_key)
}

/// Generate a random session password
fn generate_session_password() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect()
}

/// Cache session password in memory (environment variable for this process)
fn cache_session_password(password: &str) -> Result<()> {
    unsafe {
        std::env::set_var("BUNKER_SESSION_KEY", password);
    }
    Ok(())
}

/// Get cached session password from memory
fn get_cached_session_password() -> Result<String> {
    std::env::var("BUNKER_SESSION_KEY").map_err(|_| anyhow!("No cached session password"))
}

/// Parse key-value pairs from string
pub fn parse_key_value(input: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = input.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid format. Expected: key=value"));
    }

    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

/// Generate QR code
pub fn generate_qr_code(data: &str) -> Result<String> {
    use qrcode::{QrCode, render::unicode};

    let code = QrCode::new(data).map_err(|e| anyhow!("Failed to generate QR code: {}", e))?;

    let string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    Ok(string)
}

/// Clear screen
pub fn clear_screen() {
    if cfg!(windows) {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status();
    } else {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }
}
