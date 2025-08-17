use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_vault: String,
    pub editor: Option<String>,
    pub auto_sync: bool,
    pub auto_lock_minutes: Option<u64>,
    pub clipboard_timeout: u64,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub use_colors: bool,
    pub use_icons: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_vault: "default".to_string(),
            editor: None,
            auto_sync: false,
            auto_lock_minutes: Some(15),
            clipboard_timeout: 45,
            theme: Theme {
                use_colors: true,
                use_icons: true,
            },
        }
    }
}

impl Config {
    /// Load configuration
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let config_data = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_data)?;
        Ok(config)
    }

    /// Save configuration
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let config_toml = toml::to_string_pretty(self)?;
        fs::write(config_path, config_toml)?;
        Ok(())
    }

    /// Get configuration file path
    fn config_path() -> Result<PathBuf> {
        let config_dir = Storage::base_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    /// Get editor command
    pub fn editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .or_else(|| std::env::var("VISUAL").ok())
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            })
    }
}

/// Ensure configuration directory exists
pub fn ensure_config_dir() -> Result<()> {
    let base_dir = Storage::base_dir()?;
    fs::create_dir_all(&base_dir)?;
    fs::create_dir_all(base_dir.join("vaults"))?;
    fs::create_dir_all(base_dir.join("sessions"))?;
    fs::create_dir_all(base_dir.join("backups"))?;
    Ok(())
}