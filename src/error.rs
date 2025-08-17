use thiserror::Error;

#[derive(Error, Debug)]
pub enum BunkerError {
    #[error("Vault not found: {0}")]
    VaultNotFound(String),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Vault already exists: {0}")]
    VaultExists(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Session expired")]
    SessionExpired,

    #[error("No active session")]
    NoSession,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Git error: {0}")]
    GitError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for BunkerError {
    fn from(err: anyhow::Error) -> Self {
        BunkerError::Other(err.to_string())
    }
}