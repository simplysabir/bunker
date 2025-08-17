use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng as ChaChaRng},
};
use rand::{Rng, distributions::Alphanumeric};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::types::{EncryptedValue, GenerateOptions, MasterKey};

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 32;

pub struct Crypto;

impl Crypto {
    /// Derive key from password using Argon2id
    pub fn derive_key(password: &str, salt: &[u8]) -> Result<MasterKey> {
        let argon2 = Argon2::default();
        let mut key = vec![0u8; KEY_SIZE];

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;

        Ok(MasterKey::new(key))
    }

    /// Generate a new salt
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; SALT_SIZE];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        salt
    }

    /// Encrypt data with ChaCha20-Poly1305
    pub fn encrypt(data: &[u8], key: &MasterKey) -> Result<EncryptedValue> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut ChaChaRng);
        let salt = Self::generate_salt();

        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedValue {
            nonce: nonce.to_vec(),
            ciphertext,
            salt,
        })
    }

    /// Decrypt data
    pub fn decrypt(encrypted: &EncryptedValue, key: &MasterKey) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.key));
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Hash password for verification (used for session management)
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {}", e))?;

        Ok(password_hash.to_string())
    }

    /// Verify password hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash: {}", e))?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Generate secure password
    pub fn generate_password(options: &GenerateOptions) -> String {
        let mut charset = String::new();

        if let Some(custom) = &options.custom_charset {
            charset = custom.clone();
        } else {
            if options.use_lowercase {
                charset.push_str("abcdefghijklmnopqrstuvwxyz");
            }
            if options.use_uppercase {
                charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
            }
            if options.use_numbers {
                charset.push_str("0123456789");
            }
            if options.use_symbols {
                charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
            }

            // Remove ambiguous characters if requested
            if options.exclude_ambiguous {
                charset = charset.replace(&['0', 'O', 'o', 'l', '1', 'I'][..], "");
            }
        }

        if charset.is_empty() {
            // Fallback to alphanumeric
            return rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(options.length)
                .map(char::from)
                .collect();
        }

        let charset_chars: Vec<char> = charset.chars().collect();
        let mut rng = rand::thread_rng();

        (0..options.length)
            .map(|_| charset_chars[rng.gen_range(0..charset_chars.len())])
            .collect()
    }

    /// Create checksum for data integrity
    pub fn checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::default();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Encrypt with password directly (for exports)
    pub fn encrypt_with_password(
        data: &[u8],
        password: &str,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt)?;
        let encrypted = Self::encrypt(data, &key)?;

        Ok((encrypted.ciphertext, encrypted.nonce, salt))
    }

    /// Decrypt with password directly (for imports)
    pub fn decrypt_with_password(
        ciphertext: &[u8],
        nonce: &[u8],
        salt: &[u8],
        password: &str,
    ) -> Result<Vec<u8>> {
        let key = Self::derive_key(password, salt)?;
        let encrypted = EncryptedValue {
            nonce: nonce.to_vec(),
            ciphertext: ciphertext.to_vec(),
            salt: salt.to_vec(),
        };

        Self::decrypt(&encrypted, &key)
    }

    /// Generate a cryptographically secure token
    pub fn generate_token(length: usize) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        BASE64.encode(token)
    }

    /// Clear sensitive data from memory
    pub fn secure_clear(mut data: Vec<u8>) {
        data.zeroize();
    }

    /// Derive session key from user input (for encrypting master key in session)
    pub fn derive_session_key(user_input: &str, salt: &[u8]) -> Result<Vec<u8>> {
        let argon2 = Argon2::default();
        let mut key = vec![0u8; KEY_SIZE];

        argon2
            .hash_password_into(user_input.as_bytes(), salt, &mut key)
            .map_err(|e| anyhow!("Session key derivation failed: {}", e))?;

        Ok(key)
    }

    /// Encrypt master key for session storage
    pub fn encrypt_master_key_for_session(
        master_key: &MasterKey,
        session_key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(session_key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut ChaChaRng);

        let ciphertext = cipher
            .encrypt(&nonce, master_key.key.as_ref())
            .map_err(|e| anyhow!("Master key encryption failed: {}", e))?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypt master key from session storage
    pub fn decrypt_master_key_from_session(
        ciphertext: &[u8],
        nonce: &[u8],
        session_key: &[u8],
    ) -> Result<MasterKey> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(session_key));
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Master key decryption failed: {}", e))?;

        Ok(MasterKey::new(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "test_password";
        let salt = Crypto::generate_salt();
        let key = Crypto::derive_key(password, &salt).unwrap();

        let plaintext = b"secret data";
        let encrypted = Crypto::encrypt(plaintext, &key).unwrap();
        let decrypted = Crypto::decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_password_generation() {
        let options = GenerateOptions::default();
        let password = Crypto::generate_password(&options);

        assert_eq!(password.len(), options.length);
        assert!(!password.is_empty());
    }
}
