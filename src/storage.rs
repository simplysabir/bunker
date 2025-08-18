use crate::crypto::Crypto;
use crate::types::{EncryptedValue, Entry, EntryMetadata, MasterKey, Session, VaultConfig};
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use git2;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct Storage {
    vault_path: PathBuf,
    vault_name: String,
}

impl Storage {
    /// Create new storage instance
    pub fn new(vault_name: Option<String>) -> Result<Self> {
        let vault_name = match vault_name {
            Some(name) => name,
            None => {
                // Load from config, fallback to "default"
                match crate::config::Config::load() {
                    Ok(config) => config.default_vault,
                    Err(_) => "default".to_string(),
                }
            }
        };
        let vault_path = Self::vault_path(&vault_name)?;

        Ok(Self {
            vault_path,
            vault_name,
        })
    }

    /// Get vault base path
    pub fn vault_path(vault_name: &str) -> Result<PathBuf> {
        let base_dir = Self::base_dir()?;
        Ok(base_dir.join("vaults").join(vault_name))
    }

    /// Get this vault's path
    pub fn get_vault_path(&self) -> &PathBuf {
        &self.vault_path
    }

    /// Get this vault's name
    pub fn get_vault_name(&self) -> &str {
        &self.vault_name
    }

    /// Get base directory for bunker
    pub fn base_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".bunker"))
    }

    /// Initialize a new vault
    pub fn init_vault(&self, config: VaultConfig) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(&self.vault_path)?;
        fs::create_dir_all(self.vault_path.join("store"))?;

        // Save config
        let config_path = self.vault_path.join(".vault");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;

        // Initialize git if needed
        if config.git_remote.is_some() {
            git2::Repository::init(&self.vault_path)?;
        }

        Ok(())
    }

    /// Check if vault exists
    pub fn vault_exists(&self) -> bool {
        self.vault_path.exists() && self.vault_path.join(".vault").exists()
    }

    /// Check if entry exists
    pub fn entry_exists(&self, key: &str) -> Result<bool> {
        let entry_path = self.entry_path(key);
        Ok(entry_path.exists())
    }

    /// Load vault configuration
    pub fn load_config(&self) -> Result<VaultConfig> {
        let config_path = self.vault_path.join(".vault");
        let config_data = fs::read_to_string(config_path)?;
        let config: VaultConfig = serde_json::from_str(&config_data)?;
        Ok(config)
    }

    /// Save vault configuration
    pub fn save_config(&self, config: &VaultConfig) -> Result<()> {
        let config_path = self.vault_path.join(".vault");
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    /// Store an entry
    pub fn store_entry(&self, entry: &Entry, key: &MasterKey) -> Result<()> {
        // Encrypt the actual password/secret value
        let value_json = serde_json::to_vec(&entry.value)?;
        let encrypted_value = Crypto::encrypt(&value_json, key)?;

        // Create entry with encrypted value
        let stored_entry = Entry {
            value: encrypted_value,
            ..entry.clone()
        };

        // Store in filesystem
        let entry_path = self.entry_path(&entry.key);
        if let Some(parent) = entry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry_json = serde_json::to_string_pretty(&stored_entry)?;
        fs::write(entry_path, entry_json)?;

        Ok(())
    }

    /// Load an entry
    pub fn load_entry(&self, key: &str, master_key: &MasterKey) -> Result<Entry> {
        let entry_path = self.entry_path(key);
        if !entry_path.exists() {
            return Err(anyhow!("Entry '{}' not found", key));
        }

        let entry_data = fs::read_to_string(entry_path)?;
        let mut entry: Entry = serde_json::from_str(&entry_data)?;

        // Decrypt the value
        let decrypted_value = Crypto::decrypt(&entry.value, master_key)?;
        let value: EncryptedValue = serde_json::from_slice(&decrypted_value)?;
        entry.value = value;

        Ok(entry)
    }

    /// Delete an entry
    pub fn delete_entry(&self, key: &str) -> Result<()> {
        let entry_path = self.entry_path(key);
        let entry_path_clone = entry_path.clone();
        if !entry_path.exists() {
            return Err(anyhow!("Entry '{}' not found", key));
        }

        fs::remove_file(entry_path)?;

        // Clean up empty directories
        let mut parent = entry_path_clone.parent();
        while let Some(dir) = parent {
            if dir == self.vault_path.join("store") {
                break;
            }
            if fs::read_dir(dir)?.next().is_none() {
                fs::remove_dir(dir)?;
            }
            parent = dir.parent();
        }

        Ok(())
    }

    /// List all entries
    pub fn list_entries(&self) -> Result<Vec<String>> {
        let store_path = self.vault_path.join("store");
        if !store_path.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        self.walk_entries(&store_path, &store_path, &mut entries)?;
        entries.sort();
        Ok(entries)
    }

    /// Walk directory tree for entries
    fn walk_entries(&self, base: &Path, dir: &Path, entries: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_entries(base, &path, entries)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let relative = path
                    .strip_prefix(base)?
                    .to_string_lossy()
                    .replace(".json", "")
                    .replace(std::path::MAIN_SEPARATOR, "/");
                entries.push(relative);
            }
        }
        Ok(())
    }

    /// Search entries through decrypted content
    pub fn search_entries(&self, query: &str, key: &MasterKey) -> Result<Vec<(String, Entry)>> {
        let entries = self.list_entries()?;
        let mut results = Vec::new();

        for entry_key in entries {
            if let Ok(entry) = self.load_entry(&entry_key, key) {
                // Decrypt the password/value to search through it
                let decrypted_value = Crypto::decrypt(&entry.value, key)?;
                let decrypted_str = String::from_utf8(decrypted_value)?;

                // Check if query matches any of these fields:
                let query_lower = query.to_lowercase();
                let mut found_match = false;

                // 1. Entry key (filename)
                if entry_key.to_lowercase().contains(&query_lower) {
                    found_match = true;
                }

                // 2. Decrypted password/value
                if decrypted_str.to_lowercase().contains(&query_lower) {
                    found_match = true;
                }

                // 3. Username
                if let Some(username) = &entry.metadata.username {
                    if username.to_lowercase().contains(&query_lower) {
                        found_match = true;
                    }
                }

                // 4. Notes
                if let Some(notes) = &entry.metadata.notes {
                    if notes.to_lowercase().contains(&query_lower) {
                        found_match = true;
                    }
                }

                // 5. URL
                if let Some(url) = &entry.metadata.url {
                    if url.to_lowercase().contains(&query_lower) {
                        found_match = true;
                    }
                }

                // 6. Tags
                for tag in &entry.metadata.tags {
                    if tag.to_lowercase().contains(&query_lower) {
                        found_match = true;
                        break;
                    }
                }

                // 7. Custom fields
                for (field_name, field_value) in &entry.metadata.custom_fields {
                    if field_value.to_lowercase().contains(&query_lower) {
                        found_match = true;
                        break;
                    }
                }

                if found_match {
                    results.push((entry_key, entry));
                }
            }
        }

        Ok(results)
    }

    /// Get entry path
    fn entry_path(&self, key: &str) -> PathBuf {
        let safe_key = key.replace('/', std::path::MAIN_SEPARATOR_STR);
        self.vault_path
            .join("store")
            .join(format!("{}.json", safe_key))
    }

    /// Store session with encrypted master key
    pub fn store_session(&self, session: &Session) -> Result<()> {
        let session_dir = Self::base_dir()?.join("sessions");
        fs::create_dir_all(&session_dir)?;

        let session_path = session_dir.join(format!("{}.session", session.vault_name));
        let session_json = serde_json::to_string(session)?;
        fs::write(session_path, session_json)?;

        Ok(())
    }

    /// Store master key permanently (encrypted with vault-specific key)
    pub fn store_master_key_permanently(&self, master_key: &MasterKey) -> Result<()> {
        let config = self.load_config()?;

        // Use vault ID as encryption key material
        let vault_key = config.id.as_bytes();
        let salt = Crypto::generate_salt();

        // Derive encryption key from vault ID
        let encryption_key = Crypto::derive_key(&config.id.to_string(), &salt)?;

        // Encrypt master key
        let (encrypted_master_key, nonce) =
            Crypto::encrypt_master_key_for_session(master_key, &encryption_key.key)?;

        // Create permanent session (expires far in future)
        let session = Session {
            id: uuid::Uuid::new_v4(),
            vault_name: self.vault_name.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(365 * 10), // 10 years
            key_hash: config.id.to_string(),                           // Use vault ID as identifier
            encrypted_master_key,
            nonce,
            salt,
        };

        self.store_session(&session)?;
        Ok(())
    }

    /// Load master key from permanent storage
    pub fn load_master_key_permanently(&self) -> Result<MasterKey> {
        let session = self.load_session()?;
        let config = self.load_config()?;

        // Derive encryption key from vault ID
        let encryption_key = Crypto::derive_key(&config.id.to_string(), &session.salt)?;

        // Decrypt master key
        let master_key = Crypto::decrypt_master_key_from_session(
            &session.encrypted_master_key,
            &session.nonce,
            &encryption_key.key,
        )?;

        Ok(master_key)
    }

    /// Load master key from session
    pub fn load_master_key_from_session(&self, session_password: &str) -> Result<MasterKey> {
        let session = self.load_session()?;

        // Verify session password
        if !Crypto::verify_password(session_password, &session.key_hash)? {
            return Err(anyhow!("Invalid session password"));
        }

        // Derive session key and decrypt master key
        let session_key = Crypto::derive_session_key(session_password, &session.salt)?;
        let master_key = Crypto::decrypt_master_key_from_session(
            &session.encrypted_master_key,
            &session.nonce,
            &session_key,
        )?;

        Ok(master_key)
    }

    /// Load session
    pub fn load_session(&self) -> Result<Session> {
        let session_path = Self::base_dir()?
            .join("sessions")
            .join(format!("{}.session", self.vault_name));

        if !session_path.exists() {
            return Err(anyhow!("No active session"));
        }

        let session_path_clone = session_path.clone();

        let session_data = fs::read_to_string(session_path)?;
        let session: Session = serde_json::from_str(&session_data)?;

        // Check if session is expired
        if session.expires_at < Utc::now() {
            fs::remove_file(session_path_clone)?;
            return Err(anyhow!("Session expired"));
        }

        Ok(session)
    }

    /// Clear session
    pub fn clear_session(&self) -> Result<()> {
        let session_path = Self::base_dir()?
            .join("sessions")
            .join(format!("{}.session", self.vault_name));

        if session_path.exists() {
            fs::remove_file(session_path)?;
        }

        Ok(())
    }

    /// List all vaults
    pub fn list_vaults() -> Result<Vec<String>> {
        let vaults_dir = Self::base_dir()?.join("vaults");
        if !vaults_dir.exists() {
            return Ok(Vec::new());
        }

        let mut vaults = Vec::new();
        for entry in fs::read_dir(vaults_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if entry.path().join(".vault").exists() {
                        vaults.push(name.to_string());
                    }
                }
            }
        }

        vaults.sort();
        Ok(vaults)
    }

    /// Export vault
    pub fn export_vault(&self, password: &str) -> Result<Vec<u8>> {
        // Collect all entries
        let entries = self.list_entries()?;
        let mut vault_data = HashMap::new();

        for entry_key in entries {
            let entry_path = self.entry_path(&entry_key);
            let entry_data = fs::read_to_string(entry_path)?;
            vault_data.insert(entry_key, entry_data);
        }

        // Include vault config
        let config = self.load_config()?;
        let export_data = serde_json::json!({
            "version": "1.0",
            "vault_config": config,
            "entries": vault_data,
            "exported_at": Utc::now(),
        });

        let json_data = serde_json::to_vec(&export_data)?;

        // Encrypt with password
        let (ciphertext, nonce, salt) = Crypto::encrypt_with_password(&json_data, password)?;

        // Create final export
        let export = serde_json::json!({
            "bunker_export": true,
            "version": "1.0",
            "encrypted_data": BASE64.encode(&ciphertext),
            "nonce": BASE64.encode(&nonce),
            "salt": BASE64.encode(&salt),
            "checksum": Crypto::checksum(&ciphertext),
        });

        Ok(serde_json::to_vec_pretty(&export)?)
    }

    /// Import vault
    pub fn import_vault(data: &[u8], password: &str, vault_name: &str) -> Result<()> {
        let import_data: serde_json::Value = serde_json::from_slice(data)?;

        // Verify it's a bunker export
        if !import_data["bunker_export"].as_bool().unwrap_or(false) {
            return Err(anyhow!("Invalid bunker export file"));
        }

        // Decode encrypted data
        let ciphertext = BASE64.decode(
            import_data["encrypted_data"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing encrypted data"))?,
        )?;
        let nonce = BASE64.decode(
            import_data["nonce"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing nonce"))?,
        )?;
        let salt = BASE64.decode(
            import_data["salt"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing salt"))?,
        )?;

        // Verify checksum
        let checksum = import_data["checksum"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing checksum"))?;
        if Crypto::checksum(&ciphertext) != checksum {
            return Err(anyhow!("Checksum verification failed"));
        }

        // Decrypt
        let decrypted = Crypto::decrypt_with_password(&ciphertext, &nonce, &salt, password)?;
        let vault_data: serde_json::Value = serde_json::from_slice(&decrypted)?;

        // Create new vault
        let storage = Storage::new(Some(vault_name.to_string()))?;

        // Extract and save config
        let mut config: VaultConfig = serde_json::from_value(vault_data["vault_config"].clone())?;
        // Preserve the original vault ID so the KDF salt remains consistent across devices
        // This ensures the derived master key matches the one used to encrypt the entries
        config.name = vault_name.to_string();
        storage.init_vault(config)?;

        // Import entries
        if let Some(entries) = vault_data["entries"].as_object() {
            for (key, value) in entries {
                let entry_path = storage.entry_path(key);
                if let Some(parent) = entry_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(entry_path, value.as_str().unwrap_or(""))?;
            }
        }

        Ok(())
    }
}
