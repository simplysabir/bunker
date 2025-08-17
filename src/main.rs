mod cli;
mod commands;
mod config;
mod crypto;
mod error;
mod git;
mod storage;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, GitAction, VaultAction};
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle no command case
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            // Show help or interactive mode
            cli::CliDisplay::print_banner();
            println!("{}", "Welcome to Bunker! 🔐".green().bold());
            println!("Use 'bunker --help' to see available commands\n");

            // Check if default vault exists
            match storage::Storage::new(None) {
                Ok(storage) => {
                    if storage.vault_exists() {
                        println!("Default vault: {}", storage.get_vault_name().cyan());

                        // Quick access menu
                        println!("\nQuick actions:");
                        println!("  {} List passwords", "bunker list".white().bold());
                        println!("  {} Add password", "bunker add <name>".white().bold());
                        println!("  {} Get password", "bunker get <name>".white().bold());
                        println!("  {} Search passwords", "bunker search".white().bold());
                    } else {
                        println!("{}", "No vault found. Initialize one with:".yellow());
                        println!("  {}", "bunker init <vault-name>".white().bold());
                    }
                }
                Err(_) => {
                    println!("{}", "Initialize your first vault with:".yellow());
                    println!("  {}", "bunker init <vault-name>".white().bold());
                }
            }
            return Ok(());
        }
    };

    // Execute command
    match command {
        Commands::Init {
            name,
            non_interactive,
        } => commands::init::execute(name, non_interactive, cli.vault).await,

        Commands::Add {
            key,
            value,
            note,
            file,
        } => commands::add::execute(key, value, note, file, cli.vault).await,

        Commands::Get { key, copy, timeout } => commands::get::execute(key, copy, cli.vault).await,

        Commands::Edit { key, value } => commands::edit::execute(key, value, cli.vault).await,

        Commands::Remove { key, force } => commands::remove::execute(key, force, cli.vault).await,

        Commands::List { tree } => commands::list::execute(None, tree, cli.vault).await,

        Commands::Search { query } => commands::search::execute(query, cli.vault).await,

        Commands::Generate {
            length,
            uppercase,
            lowercase,
            numbers,
            symbols,
            no_ambiguous,
            charset,
            copy,
        } => {
            let options = types::GenerateOptions {
                length,
                use_uppercase: uppercase,
                use_lowercase: lowercase,
                use_numbers: numbers,
                use_symbols: symbols,
                exclude_ambiguous: no_ambiguous,
                custom_charset: charset,
            };
            commands::generate::execute(None, length, !symbols, !numbers, !uppercase, cli.vault)
                .await
        }

        Commands::Copy { key, timeout } => {
            commands::copy::execute(key, false, timeout, cli.vault).await
        }

        Commands::Peek { key, chars } => commands::peek::execute(key, cli.vault).await,

        Commands::Move { from, to } => commands::move_cmd::execute(from, to, cli.vault).await,

        Commands::Exec { command, key, env } => {
            commands::exec::execute(command, key, env, cli.vault).await
        }

        Commands::Export {
            format,
            output,
            metadata,
        } => commands::export::execute(format, output, metadata, cli.vault).await,

        Commands::Import {
            file,
            format,
            overwrite,
        } => commands::import::execute(file, format, overwrite, cli.vault).await,

        Commands::Grep {
            pattern,
            ignore_case,
        } => commands::grep::execute(pattern, ignore_case, cli.vault).await,

        Commands::Git { action } => match action {
            GitAction::Sync => commands::sync::execute(None, cli.vault).await,
            GitAction::Pull => commands::pull::execute(cli.vault).await,
            GitAction::Status => commands::status::execute(cli.vault).await,
            GitAction::Restore { commit, key } => {
                commands::restore::execute(commit, key, cli.vault).await
            }
        },

        Commands::Vault { action } => match action {
            VaultAction::Create { name } => {
                commands::vault::execute(cli::VaultAction::Create { name }).await
            }
            VaultAction::Use { name } => {
                commands::vault::execute(cli::VaultAction::Use { name }).await
            }
            VaultAction::List => commands::vault::execute(cli::VaultAction::List).await,
            VaultAction::Delete { name, force } => {
                commands::vault::execute(cli::VaultAction::Delete { name, force }).await
            }
            VaultAction::Export { password, output } => {
                commands::export_vault::execute(password, output, cli.vault).await
            }
            VaultAction::Import {
                file,
                password,
                name,
            } => commands::import_vault::execute(file, password, name).await,
        },

        Commands::Lock => commands::lock::execute(cli.vault).await,

        Commands::Unlock { duration } => commands::unlock::execute(cli.vault, Some(duration)).await,

        Commands::Status => commands::status::execute(cli.vault).await,

        Commands::Backup { destination } => commands::backup::execute(destination, cli.vault).await,

        Commands::Restore { backup, vault_name } => {
            commands::restore_backup::execute(backup, vault_name).await
        }

        Commands::History { key, limit } => {
            commands::history::execute(key, Some(limit), cli.vault).await
        }

        Commands::Env { key, var } => commands::env::execute(key, var, cli.vault).await,
    }
}
