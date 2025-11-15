use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use dirs::home_dir;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub run_counter: u64,
    pub last_run_time: Option<u64>,
}

impl State {
    pub fn load() -> Result<Self> {
        let state_dir = get_state_dir()?;
        let state_file = state_dir.join("state.json.enc");
        
        if !state_file.exists() {
            return Ok(State {
                run_counter: 0,
                last_run_time: None,
            });
        }
        
        // Read encrypted state
        let encrypted_data = fs::read(&state_file)
            .context("Failed to read state file")?;
        
        // Decrypt state (using machine-specific key)
        let decrypted = decrypt_state(&encrypted_data)?;
        
        // Verify checksum
        let (data, checksum) = split_checksum(&decrypted)?;
        verify_checksum(data, &checksum)?;
        
        let state: State = serde_json::from_slice(data)
            .context("Failed to parse state JSON")?;
        
        Ok(state)
    }
    
    pub fn save(&self) -> Result<()> {
        let state_dir = get_state_dir()?;
        fs::create_dir_all(&state_dir)
            .context("Failed to create state directory")?;
        
        let state_file = state_dir.join("state.json.enc");
        
        // Serialize state
        let json_data = serde_json::to_vec(self)
            .context("Failed to serialize state")?;
        
        // Add checksum
        let checksum = calculate_checksum(&json_data);
        let mut data_with_checksum = json_data;
        data_with_checksum.extend_from_slice(&checksum);
        
        // Encrypt state
        let encrypted = encrypt_state(&data_with_checksum)?;
        
        // Write encrypted state
        fs::write(&state_file, encrypted)
            .context("Failed to write state file")?;
        
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&state_file)?.permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&state_file, perms)?;
        }
        
        Ok(())
    }
    
    pub fn increment_counter(&mut self) {
        self.run_counter += 1;
        self.last_run_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }
}

fn get_state_dir() -> Result<PathBuf> {
    let home = home_dir()
        .context("Failed to get home directory")?;
    Ok(home.join(".exam-recorder"))
}

fn calculate_checksum(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn verify_checksum(data: &[u8], expected: &[u8]) -> Result<()> {
    let calculated = calculate_checksum(data);
    if calculated.as_slice() != expected {
        anyhow::bail!("State checksum verification failed - possible tampering");
    }
    Ok(())
}

fn split_checksum(data: &[u8]) -> Result<(&[u8], &[u8])> {
    if data.len() < 32 {
        anyhow::bail!("Invalid state data length");
    }
    let (payload, checksum) = data.split_at(data.len() - 32);
    Ok((payload, checksum))
}

fn encrypt_state(data: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::RngCore;
    
    let key = derive_state_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let mut ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
    
    // Prepend nonce
    let mut result = nonce_bytes.to_vec();
    result.append(&mut ciphertext);
    
    Ok(result)
}

fn decrypt_state(encrypted: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    if encrypted.len() < 12 {
        anyhow::bail!("Invalid encrypted data length");
    }
    
    let key = derive_state_key()?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let ciphertext = &encrypted[12..];
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

fn derive_state_key() -> Result<aes_gcm::Key<aes_gcm::Aes256Gcm>> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    
    // Use machine-specific information
    if let Ok(hostname) = hostname::get() {
        if let Some(hostname_str) = hostname.to_str() {
            hasher.update(hostname_str.as_bytes());
        }
    }
    
    // Add user-specific data
    if let Ok(username) = std::env::var("USER") {
        hasher.update(username.as_bytes());
    }
    
    // Add a constant salt (hardcoded in binary)
    hasher.update(b"exam-recorder-state-key-v1");
    
    let hash = hasher.finalize();
    Ok(*aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(&hash))
}

