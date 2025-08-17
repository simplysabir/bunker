use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Main entry stored in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub key: String,
    pub value: EncryptedValue,
    pub metadata: EntryMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: Option<DateTime<Utc>>,
}

/// Encrypted value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedValue {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub salt: Vec<u8>,
}

/// Entry metadata (stored separately, can be encrypted)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntryMetadata {
    pub entry_type: EntryType,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub username: Option<String>,
    pub custom_fields: HashMap<String, String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_type: Option<String>,
}

/// Type of entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Password,
    Note,
    Card,
    Identity,
    SecureFile,
    ApiKey,
    SshKey,
    Database,
    Custom(String),
}

impl Default for EntryType {
    fn default() -> Self {
        Self::Password
    }
}

/// Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub encryption: EncryptionConfig,
    pub git_remote: Option<String>,
    pub auto_sync: bool,
    pub auto_lock_minutes: Option<u64>,
}

/// Encryption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,  // "chacha20poly1305"
    pub kdf: String,        // "argon2id"
    pub kdf_iterations: u32,
    pub kdf_memory: u32,
    pub kdf_parallelism: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: "chacha20poly1305".to_string(),
            kdf: "argon2id".to_string(),
            kdf_iterations: 3,
            kdf_memory: 65536,  // 64 MB
            kdf_parallelism: 2,
        }
    }
}

/// Session information (for unlock/lock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub vault_name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub key_hash: String,  // For verification
}

/// Master key wrapper (zeroized on drop)
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    pub key: Vec<u8>,
}

impl MasterKey {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

/// Vault export format
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultExport {
    pub version: String,
    pub vault_config: VaultConfig,
    pub encrypted_data: Vec<u8>,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub checksum: String,
    pub exported_at: DateTime<Utc>,
}

/// Import/Export entry format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub key: String,
    pub value: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub key: String,
    pub entry_type: EntryType,
    pub username: Option<String>,
    pub url: Option<String>,
    pub score: f32,  // Relevance score
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub commit_hash: String,
    pub key: String,
    pub action: HistoryAction,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryAction {
    Created,
    Updated,
    Deleted,
    Renamed,
}

/// CLI display theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub success_prefix: String,
    pub error_prefix: String,
    pub warning_prefix: String,
    pub info_prefix: String,
    pub tree_chars: TreeChars,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeChars {
    pub vertical: String,
    pub horizontal: String,
    pub branch: String,
    pub last_branch: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            success_prefix: "✓".to_string(),
            error_prefix: "✗".to_string(),
            warning_prefix: "⚠".to_string(),
            info_prefix: "→".to_string(),
            tree_chars: TreeChars {
                vertical: "│".to_string(),
                horizontal: "─".to_string(),
                branch: "├──".to_string(),
                last_branch: "└──".to_string(),
            },
        }
    }
}

/// Password generation options
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub length: usize,
    pub use_uppercase: bool,
    pub use_lowercase: bool,
    pub use_numbers: bool,
    pub use_symbols: bool,
    pub exclude_ambiguous: bool,
    pub custom_charset: Option<String>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            length: 20,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
            exclude_ambiguous: true,
            custom_charset: None,
        }
    }
}