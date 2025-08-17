use anyhow::Result;
use colored::*;

use crate::crypto::Crypto;
use crate::types::GenerateOptions;

pub async fn execute(
    key: Option<String>,
    length: usize,
    no_symbols: bool,
    no_numbers: bool,
    no_uppercase: bool,
    vault: Option<String>,
) -> Result<()> {
    let options = GenerateOptions {
        length,
        use_uppercase: !no_uppercase,
        use_lowercase: true,
        use_numbers: !no_numbers,
        use_symbols: !no_symbols,
        exclude_ambiguous: true,
        custom_charset: None,
    };
    
    let password = Crypto::generate_password(&options);
    
    if let Some(k) = key {
        // Save the generated password
        super::add::execute(k.clone(), Some(password.clone()), false, None, vault).await?;
        println!("{} Generated and saved password for '{}'", "✓".green().bold(), k.cyan());
        println!("Password: {}", password.yellow());
    } else {
        // Just display the password
        println!("{}", password.green().bold());
    }
    
    Ok(())
}