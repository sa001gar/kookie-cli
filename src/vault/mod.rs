//! Vault module for managing encrypted storage

pub mod storage;
pub mod types;

use crate::crypto::{self, kdf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use types::*;

/// Vault errors
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault not initialized. Run 'kookie init' first.")]
    NotInitialized,
    #[error("Vault already exists. Use --force to reinitialize.")]
    AlreadyExists,
    #[error("Wrong master password")]
    WrongPassword,
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
    #[error("Duplicate secret name: {0}")]
    DuplicateName(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Key derivation error: {0}")]
    KdfError(#[from] kdf::KdfError),
}

/// Encrypted vault file format
#[derive(Serialize, Deserialize)]
pub struct VaultFile {
    pub version: u32,
    pub salt: String,
    pub encrypted_data: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Decrypted vault contents
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct VaultData {
    pub passwords: Vec<Password>,
    pub api_keys: Vec<ApiKey>,
    pub notes: Vec<Note>,
    pub db_credentials: Vec<DbCredential>,
    pub tokens: Vec<Token>,
}

/// Main vault structure
pub struct Vault {
    pub path: PathBuf,
    pub data: VaultData,
    key: Option<[u8; 32]>,
    salt: String,
}

impl Vault {
    /// Creates a new vault at the default location
    pub fn new() -> Self {
        Self {
            path: storage::get_vault_path(),
            data: VaultData::default(),
            key: None,
            salt: String::new(),
        }
    }

    /// Checks if vault exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Initializes a new vault with the given master password
    pub fn init(&mut self, master_password: &str) -> Result<(), VaultError> {
        if self.exists() {
            return Err(VaultError::AlreadyExists);
        }

        // Generate salt and derive key
        self.salt = kdf::generate_salt();
        self.key = Some(kdf::derive_key(master_password, &self.salt)?);
        self.data = VaultData::default();

        // Save the vault
        self.save()?;

        Ok(())
    }

    /// Initializes a new vault, forcing overwrite if exists
    pub fn init_force(&mut self, master_password: &str) -> Result<(), VaultError> {
        // Generate salt and derive key
        self.salt = kdf::generate_salt();
        self.key = Some(kdf::derive_key(master_password, &self.salt)?);
        self.data = VaultData::default();

        // Save the vault
        self.save()?;

        Ok(())
    }

    /// Unlocks the vault with the master password
    pub fn unlock(&mut self, master_password: &str) -> Result<(), VaultError> {
        if !self.exists() {
            return Err(VaultError::NotInitialized);
        }

        // Load vault file
        let vault_file = storage::load_vault_file(&self.path)?;
        self.salt = vault_file.salt.clone();

        // Derive key
        let key = kdf::derive_key(master_password, &vault_file.salt)?;

        // Try to decrypt
        let decrypted = crypto::decrypt(&key, &vault_file.encrypted_data)
            .map_err(|_| VaultError::WrongPassword)?;

        // Deserialize
        self.data = serde_json::from_slice(&decrypted)?;
        self.key = Some(key);

        Ok(())
    }

    /// Checks if vault is unlocked
    #[allow(dead_code)]
    pub fn is_unlocked(&self) -> bool {
        self.key.is_some()
    }

    /// Locks the vault (clears key from memory)
    #[allow(dead_code)]
    pub fn lock(&mut self) {
        self.key = None;
    }

    /// Saves the vault to disk
    pub fn save(&self) -> Result<(), VaultError> {
        let key = self.key.ok_or(VaultError::WrongPassword)?;

        // Serialize data
        let data_json = serde_json::to_vec(&self.data)?;

        // Encrypt
        let encrypted = crypto::encrypt(&key, &data_json)
            .map_err(|e| VaultError::EncryptionError(e.to_string()))?;

        // Create vault file
        let vault_file = VaultFile {
            version: 1,
            salt: self.salt.clone(),
            encrypted_data: encrypted,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        // Save
        storage::save_vault_file(&self.path, &vault_file)?;

        Ok(())
    }

    // === Password Operations ===

    pub fn add_password(&mut self, password: Password) -> Result<(), VaultError> {
        if self.data.passwords.iter().any(|p| p.name == password.name) {
            return Err(VaultError::DuplicateName(password.name));
        }
        self.data.passwords.push(password);
        self.save()
    }

    pub fn get_password(&self, id_or_name: &str) -> Option<&Password> {
        self.data.passwords.iter().find(|p| p.id == id_or_name || p.name == id_or_name)
    }

    pub fn delete_password(&mut self, id_or_name: &str) -> Result<Password, VaultError> {
        let idx = self.data.passwords.iter()
            .position(|p| p.id == id_or_name || p.name == id_or_name)
            .ok_or_else(|| VaultError::SecretNotFound(id_or_name.to_string()))?;
        let removed = self.data.passwords.remove(idx);
        self.save()?;
        Ok(removed)
    }

    // === API Key Operations ===

    pub fn add_api_key(&mut self, api_key: ApiKey) -> Result<(), VaultError> {
        if self.data.api_keys.iter().any(|k| k.name == api_key.name) {
            return Err(VaultError::DuplicateName(api_key.name));
        }
        self.data.api_keys.push(api_key);
        self.save()
    }

    pub fn get_api_key(&self, id_or_name: &str) -> Option<&ApiKey> {
        self.data.api_keys.iter().find(|k| k.id == id_or_name || k.name == id_or_name)
    }

    pub fn delete_api_key(&mut self, id_or_name: &str) -> Result<ApiKey, VaultError> {
        let idx = self.data.api_keys.iter()
            .position(|k| k.id == id_or_name || k.name == id_or_name)
            .ok_or_else(|| VaultError::SecretNotFound(id_or_name.to_string()))?;
        let removed = self.data.api_keys.remove(idx);
        self.save()?;
        Ok(removed)
    }

    // === Note Operations ===

    pub fn add_note(&mut self, note: Note) -> Result<(), VaultError> {
        if self.data.notes.iter().any(|n| n.name == note.name) {
            return Err(VaultError::DuplicateName(note.name));
        }
        self.data.notes.push(note);
        self.save()
    }

    pub fn get_note(&self, id_or_name: &str) -> Option<&Note> {
        self.data.notes.iter().find(|n| n.id == id_or_name || n.name == id_or_name)
    }

    pub fn delete_note(&mut self, id_or_name: &str) -> Result<Note, VaultError> {
        let idx = self.data.notes.iter()
            .position(|n| n.id == id_or_name || n.name == id_or_name)
            .ok_or_else(|| VaultError::SecretNotFound(id_or_name.to_string()))?;
        let removed = self.data.notes.remove(idx);
        self.save()?;
        Ok(removed)
    }

    // === DB Credential Operations ===

    pub fn add_db_credential(&mut self, cred: DbCredential) -> Result<(), VaultError> {
        if self.data.db_credentials.iter().any(|c| c.name == cred.name) {
            return Err(VaultError::DuplicateName(cred.name));
        }
        self.data.db_credentials.push(cred);
        self.save()
    }

    pub fn get_db_credential(&self, id_or_name: &str) -> Option<&DbCredential> {
        self.data.db_credentials.iter().find(|c| c.id == id_or_name || c.name == id_or_name)
    }

    pub fn delete_db_credential(&mut self, id_or_name: &str) -> Result<DbCredential, VaultError> {
        let idx = self.data.db_credentials.iter()
            .position(|c| c.id == id_or_name || c.name == id_or_name)
            .ok_or_else(|| VaultError::SecretNotFound(id_or_name.to_string()))?;
        let removed = self.data.db_credentials.remove(idx);
        self.save()?;
        Ok(removed)
    }

    // === Token Operations ===

    pub fn add_token(&mut self, token: Token) -> Result<(), VaultError> {
        if self.data.tokens.iter().any(|t| t.name == token.name) {
            return Err(VaultError::DuplicateName(token.name));
        }
        self.data.tokens.push(token);
        self.save()
    }

    pub fn get_token(&self, id_or_name: &str) -> Option<&Token> {
        self.data.tokens.iter().find(|t| t.id == id_or_name || t.name == id_or_name)
    }

    pub fn delete_token(&mut self, id_or_name: &str) -> Result<Token, VaultError> {
        let idx = self.data.tokens.iter()
            .position(|t| t.id == id_or_name || t.name == id_or_name)
            .ok_or_else(|| VaultError::SecretNotFound(id_or_name.to_string()))?;
        let removed = self.data.tokens.remove(idx);
        self.save()?;
        Ok(removed)
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}
