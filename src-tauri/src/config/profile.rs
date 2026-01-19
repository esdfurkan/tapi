use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub api_key: Option<String>,
    pub language: String,
    pub model: Option<String>,
    pub font: Option<String>,
    pub theme: String,
    #[serde(default)]
    pub total_credits_used: u64,
    // Custom API Endpoints
    #[serde(default = "default_storage_url")]
    pub storage_url: String,
    #[serde(default = "default_ocr_url")]
    pub ocr_url: String,
    #[serde(default = "default_translate_url")]
    pub translate_url: String,
    // Storage parameters
    #[serde(default)]
    pub storage_urls: String,
    
    // Custom Headers & Debugging
    #[serde(default)]
    pub storage_headers: String, // JSON text
    #[serde(default)]
    pub ocr_headers: String,     // JSON text
    #[serde(default)]
    pub save_debug_json: bool,
}

fn default_storage_url() -> String {
    "https://api.toriitranslate.com/api/storage".to_string()
}

fn default_ocr_url() -> String {
    "https://api.toriitranslate.com/api/ocr".to_string()
}

fn default_translate_url() -> String {
    "https://api.toriitranslate.com/api/upload".to_string()
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            api_key: None,
            language: "en".to_string(),
            model: None,
            font: Some("wildwords".to_string()),
            theme: "dark".to_string(),
            total_credits_used: 0,
            storage_url: default_storage_url(),
            ocr_url: default_ocr_url(),
            translate_url: default_translate_url(),
            storage_urls: String::new(),
            storage_headers: String::new(),
            ocr_headers: String::new(),
            save_debug_json: false,
        }
    }
}

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce
};

impl Profile {
    // Helper: Retrieves a unique identifier for the machine.
    // 1. Tries to get the Hardware Machine ID (Best).
    // 2. Falls back to Hostname (Soft Binding).
    // 3. Last resort is a generic string (but unlikely to reach here).
    fn get_device_fingerprint() -> String {
        #[cfg(target_os = "android")]
        {
            // Android doesn't support machine-uid crate.
            // Using hostname as a soft-bind fallback for mobile.
            sysinfo::System::host_name().unwrap_or_else(|| "android-device-soft-id".to_string())
        }

        #[cfg(not(target_os = "android"))]
        {
            machine_uid::get().unwrap_or_else(|_| {
                sysinfo::System::host_name().unwrap_or_else(|| "generic-device".to_string())
            })
        }
    }

    // Dynamic prefix generation based on Machine UID.
    // The file will not have a static prefix anymore.
    // Generates a 16-character (8 byte) random-looking hex tag derived from the machine UID.
    fn get_prefix() -> String {
        let uid = Self::get_device_fingerprint();
        // Hash the UID with a domain tag for the prefix
        let hash = blake3::hash(format!("prefix-tag|{}", uid).as_bytes());
        // Use the first 16 characters (8 bytes) of the hex hash as the prefix
        let hex_tag = hex::encode(&hash.as_bytes()[0..8]).to_uppercase();
        format!("{}::", hex_tag)
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let mut profile: Profile = serde_json::from_str(&content)?;
        
        // Decrypt API key if encrypted
        if let Some(key) = &profile.api_key {
            let prefix = Self::get_prefix();
            // Only check for machine-specific prefix. Legacy support removed.
            if key.starts_with(&prefix) {
                profile.api_key = Some(Self::decrypt(key));
            }
        }
        
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut to_save = self.clone();
        
        // Encrypt API key (with error handling)
        if let Some(key) = &to_save.api_key {
            if !key.is_empty() {
                let prefix = Self::get_prefix();
                if !key.starts_with(&prefix) {
                    to_save.api_key = Some(Self::encrypt(key));
                } else {
                    // Re-encrypt to refresh nonce
                    let plain = Self::decrypt(key);
                    to_save.api_key = Some(Self::encrypt(&plain));
                }
            }
        }

        let content = serde_json::to_string_pretty(&to_save)?;
        
        // Atomic write: write to temp file first, then rename
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, path)?;
        
        Ok(())
    }

    // Derives a hardware-bound 32-byte key using Blake3 + XChaCha20Poly1305.
    // This creates a key derived from: HardwareID -> Blake3 -> XChaCha20 -> Blake3 -> FinalKey
    fn get_machine_key() -> chacha20poly1305::Key {
        let uid = Self::get_device_fingerprint();
        
        // 1. Initial Seed from Hardware (Blake3)
        let seed_hash = blake3::hash(uid.as_bytes());
        let seed_key = chacha20poly1305::Key::from_slice(seed_hash.as_bytes());
        
        // 2. Use XChaCha20 to mix the seed into a new derived key.
        let cipher = XChaCha20Poly1305::new(seed_key);
        // Deterministic nonce for key derivation (Safe here because key is unique per device)
        let nonce = XNonce::from_slice(&[0u8; 24]); 
        
        // Encrypt the machine UID itself to generate cryptographic derived bytes.
        // This ensures NO static strings are used in the key generation process.
        // The key is derived from: Encrypt(Key=Hash(UID), Plaintext=UID).
        let derived = cipher.encrypt(nonce, uid.as_bytes()).unwrap_or_else(|_| seed_hash.as_bytes().to_vec());
        
        // 3. Finalize to 32 bytes
        let final_hash = blake3::hash(&derived);
        *chacha20poly1305::Key::from_slice(final_hash.as_bytes())
    }

    fn encrypt(text: &str) -> String {
        // Try to encrypt, but fallback gracefully on any error
        match Self::try_encrypt(text) {
            Ok(encrypted) => encrypted,
            Err(e) => {
                eprintln!("Encryption failed: {}, storing plaintext", e);
                text.to_string()
            }
        }
    }

    fn try_encrypt(text: &str) -> Result<String> {
        let key = Self::get_machine_key();
        let cipher = XChaCha20Poly1305::new(&key);

        // Generate a random 24-byte nonce using time + process info
        let time_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Time error: {}", e))?
            .as_nanos();
            
        let nonce_hash = blake3::hash(&time_seed.to_le_bytes());
        let nonce_bytes = &nonce_hash.as_bytes()[0..24];
        let nonce = XNonce::from_slice(nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, text.as_bytes())
            .map_err(|e| anyhow::anyhow!("Cipher error: {}", e))?;
        
        let mut final_payload = Vec::with_capacity(24 + ciphertext.len());
        final_payload.extend_from_slice(nonce_bytes);
        final_payload.extend_from_slice(&ciphertext);

        Ok(format!("{}{}", Self::get_prefix(), general_purpose::STANDARD.encode(final_payload)))
    }

    fn decrypt(text: &str) -> String {
        let prefix = Self::get_prefix();
        if !text.starts_with(&prefix) { return text.to_string(); }
        
        let encoded_data = &text[prefix.len()..];
        
        if let Ok(data) = general_purpose::STANDARD.decode(encoded_data) {
            // Check length: Nonce (24) + Tag (16) = 40 bytes minimum overhead
            if data.len() < 24 + 16 { return text.to_string(); } 
            
            let (nonce_bytes, ciphertext) = data.split_at(24);
            let nonce = XNonce::from_slice(nonce_bytes);
            
            let key = Self::get_machine_key();
            let cipher = XChaCha20Poly1305::new(&key);
            
            if let Ok(decrypted_bytes) = cipher.decrypt(nonce, ciphertext) {
                return String::from_utf8(decrypted_bytes).unwrap_or(text.to_string());
            }
        }
        
        // Decryption failed (wrong key/machine or tampered data)
        text.to_string()
    }
}
