use anyhow::Result;
use colored::*;

use crate::storage::Storage;

pub async fn execute(vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault)?;
    
    // Clear session
    storage.clear_session()?;
    
    // Clear cached session password
    unsafe {
        std::env::remove_var("BUNKER_SESSION_KEY");
    }
    
    println!("{} Vault locked successfully", "🔒".green().bold());
    println!("You'll need to enter your password again to access the vault");
    
    Ok(())
}
