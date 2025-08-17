use anyhow::{anyhow, Result};
use colored::*;
use skim::prelude::*;
use std::io::Cursor;

use crate::storage::Storage;
use crate::utils;

pub async fn execute(query: Option<String>, vault: Option<String>) -> Result<()> {
    let storage = Storage::new(vault.clone())?;
    
    if !storage.vault_exists() {
        return Err(anyhow!("Vault not initialized. Run 'bunker init' first"));
    }
    
    // Get all entries
    let entries = storage.list_entries()?;
    
    if entries.is_empty() {
        println!("{}", "No passwords stored yet".yellow());
        return Ok(());
    }
    
    if let Some(q) = query {
        // Search with provided query
        let results: Vec<String> = entries
            .into_iter()
            .filter(|e| e.to_lowercase().contains(&q.to_lowercase()))
            .collect();
        
        if results.is_empty() {
            println!("{}", "No matches found".yellow());
        } else {
            println!("{} Found {} matches:\n", "🔍".green(), results.len().to_string().bold());
            for result in results {
                println!("  {}", result.cyan());
            }
        }
    } else {
        // Interactive fuzzy search
        let options = SkimOptionsBuilder::default()
            .prompt(Some("Search: "))
            .preview(Some(""))
            .build()
            .unwrap();
        
        let input = entries.join("\n");
        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(input));
        
        let selected = Skim::run_with(&options, Some(items))
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);
        
        if !selected.is_empty() {
            let choice = selected[0].output().to_string();
            
            // Ask what to do with the selected entry
            println!("\nSelected: {}", choice.cyan().bold());
            println!("\nWhat would you like to do?");
            println!("  {} Copy password", "1.".blue());
            println!("  {} Show password", "2.".blue());
            println!("  {} Edit entry", "3.".blue());
            println!("  {} Cancel", "4.".blue());
            
            let action = utils::prompt_input("Choice (1-4)")?;
            
            match action.as_str() {
                "1" => super::copy::execute(choice.clone(), false, 45, vault.clone()).await?,
                "2" => super::get::execute(choice.clone(), false, vault.clone()).await?,
                "3" => super::edit::execute(choice, None, vault).await?,
                _ => println!("Cancelled"),
            }
        }
    }
    
    Ok(())
}