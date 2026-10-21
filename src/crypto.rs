use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct SessionCrypto {
    key: [u8; 32],
}

impl SessionCrypto {
    /// Derive a 256-bit encryption key from device ID and PIN or passphrase
    pub fn new_from_credentials(device_id: &str, pin: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"OxiDesk-v2-Salt-KeyDerivation:");
        hasher.update(device_id.trim().as_bytes());
        hasher.update(b":");
        hasher.update(pin.trim().as_bytes());
        let result = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        Self { key }
    }

    /// Direct key initialization
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt plaintext using AES-256-GCM.
    /// Format: [12 bytes Nonce] + [Ciphertext + 16 bytes Poly1305 Tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| format!("Cipher init error: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption error: {}", e))?;

        let mut output = Vec::with_capacity(12 + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypt ciphertext using AES-256-GCM.
    pub fn decrypt(&self, payload: &[u8]) -> Result<Vec<u8>, String> {
        if payload.len() < 12 + 16 {
            return Err("Payload too short for AES-GCM".to_string());
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| format!("Cipher init error: {}", e))?;

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_roundtrip() {
        let crypto = SessionCrypto::new_from_credentials("123456789", "8821");
        let secret_data = b"OxiDesk Remote Desktop Frame / Command Data";

        let encrypted = crypto.encrypt(secret_data).expect("Encryption should succeed");
        assert_ne!(encrypted, secret_data);

        let decrypted = crypto.decrypt(&encrypted).expect("Decryption should succeed");
        assert_eq!(decrypted, secret_data);
    }
}
