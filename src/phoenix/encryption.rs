use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Handles AES-256-GCM encryption/decryption for Phoenix Vault
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    /// Create a new encryptor with the given 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        Self { cipher }
    }

    /// Encrypt data with AES-256-GCM
    /// Returns nonce (12 bytes) + ciphertext
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Generate random 12-byte nonce
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Combine nonce + ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data with AES-256-GCM
    /// Expects input format: nonce (12 bytes) + ciphertext
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if encrypted.len() < 12 {
            return Err(EncryptionError::InvalidData(
                "Data too short to contain nonce".into()
            ));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }

    /// Encrypt a file's contents
    pub async fn encrypt_file(
        &self,
        source_path: &std::path::Path,
        dest_path: &std::path::Path,
    ) -> Result<u64, EncryptionError> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        // Read source file
        let plaintext = fs::read(source_path)
            .await
            .map_err(|e| EncryptionError::InvalidData(format!("Failed to read source file: {}", e)))?;

        // Encrypt
        let encrypted = self.encrypt(&plaintext)?;

        // Write to destination
        let mut file = fs::File::create(dest_path)
            .await
            .map_err(|e| EncryptionError::EncryptionFailed(format!("Failed to create dest file: {}", e)))?;

        file.write_all(&encrypted)
            .await
            .map_err(|e| EncryptionError::EncryptionFailed(format!("Failed to write encrypted data: {}", e)))?;

        Ok(encrypted.len() as u64)
    }

    /// Decrypt a file's contents
    pub async fn decrypt_file(
        &self,
        source_path: &std::path::Path,
        dest_path: &std::path::Path,
    ) -> Result<u64, EncryptionError> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        // Read encrypted file
        let encrypted = fs::read(source_path)
            .await
            .map_err(|e| EncryptionError::InvalidData(format!("Failed to read encrypted file: {}", e)))?;

        // Decrypt
        let decrypted = self.decrypt(&encrypted)?;

        // Write to destination
        let mut file = fs::File::create(dest_path)
            .await
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Failed to create dest file: {}", e)))?;

        file.write_all(&decrypted)
            .await
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Failed to write decrypted data: {}", e)))?;

        Ok(decrypted.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_decryption() {
        let key = [0u8; 32];
        let encryptor = Encryptor::new(&key);

        let plaintext = b"Hello, world!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
        assert!(encrypted.len() > plaintext.len()); // Should include nonce
    }

    #[tokio::test]
    async fn test_file_encryption() {
        let key = [0u8; 32];
        let encryptor = Encryptor::new(&key);

        // Create temp files
        let source = NamedTempFile::new().unwrap();
        let encrypted = NamedTempFile::new().unwrap();
        let decrypted = NamedTempFile::new().unwrap();

        // Write test data
        tokio::fs::write(source.path(), b"Test data").await.unwrap();

        // Encrypt
        encryptor.encrypt_file(
            source.path(),
            encrypted.path()
        ).await.unwrap();

        // Decrypt
        encryptor.decrypt_file(
            encrypted.path(),
            decrypted.path()
        ).await.unwrap();

        // Verify
        let result = tokio::fs::read(decrypted.path()).await.unwrap();
        assert_eq!(result, b"Test data");
    }

    #[test]
    fn test_invalid_data() {
        let key = [0u8; 32];
        let encryptor = Encryptor::new(&key);

        // Too short to contain nonce
        let result = encryptor.decrypt(&[1, 2, 3]);
        assert!(matches!(result, Err(EncryptionError::InvalidData(_))));

        // Invalid ciphertext
        let result = encryptor.decrypt(&[0u8; 20]);
        assert!(matches!(result, Err(EncryptionError::DecryptionFailed(_))));
    }
}