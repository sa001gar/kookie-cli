//! Key and secret generators

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;

/// Generates a random key of specified length
pub fn generate_random_key(length: usize) -> String {
    let mut bytes = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Generates a random key suitable for JWT secrets (256 bits)
pub fn generate_jwt_secret() -> String {
    generate_random_key(32) // 256 bits
}

/// Generates a random key suitable for encryption (256 bits)
#[allow(dead_code)]
pub fn generate_encryption_key() -> String {
    generate_random_key(32)
}

/// Generates a random API key
pub fn generate_api_key() -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("kk_{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// Generates a secure random password
pub fn generate_password(length: usize, include_symbols: bool) -> String {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const DIGITS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let charset: Vec<u8> = if include_symbols {
        [LETTERS, DIGITS, SYMBOLS].concat()
    } else {
        [LETTERS, DIGITS].concat()
    };
    
    let mut password = Vec::with_capacity(length);
    let mut rng = rand::thread_rng();
    
    for _ in 0..length {
        let idx = (rng.next_u32() as usize) % charset.len();
        password.push(charset[idx]);
    }
    
    String::from_utf8(password).unwrap_or_else(|_| generate_random_key(length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_key_length() {
        let key = generate_random_key(32);
        // Base64 encoded 32 bytes should be about 43 characters
        assert!(key.len() >= 40);
    }

    #[test]
    fn test_generate_jwt_secret() {
        let secret = generate_jwt_secret();
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_generate_api_key_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("kk_"));
    }

    #[test]
    fn test_generate_password_length() {
        let password = generate_password(16, true);
        assert_eq!(password.len(), 16);
    }
}
