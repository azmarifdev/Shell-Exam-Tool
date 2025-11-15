use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::analyzer::DecryptedData;

pub struct Decryptor {
    zip_path: std::path::PathBuf,
}

impl Decryptor {
    pub fn new<P: AsRef<Path>>(zip_path: P) -> Result<Self> {
        let path = zip_path.as_ref().to_path_buf();
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }
        Ok(Decryptor { zip_path: path })
    }
    
    pub fn decrypt(&self, password: &str) -> Result<DecryptedData> {
        // Read encrypted ZIP
        let encrypted_zip = fs::read(&self.zip_path)
            .context("Failed to read ZIP file")?;
        
        // Decrypt ZIP
        let zip_data = decrypt_file(&encrypted_zip, password)
            .context("Failed to decrypt ZIP file")?;
        
        // Extract files from ZIP
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data))
            .context("Failed to open ZIP archive")?;
        
        let mut events_json = None;
        let mut summary_json = None;
        let mut metadata_json = None;
        let mut terminal_output = None;
        let mut state_copy_json = None;
        let mut integrity_hash = None;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read ZIP entry")?;
            
            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents)
                .context("Failed to read ZIP file contents")?;
            
            let name = file.name().to_string();
            
            match name.as_str() {
                "events.json.enc" => {
                    let decrypted = decrypt_file(&contents, password)?;
                    events_json = Some(serde_json::from_slice(&decrypted)?);
                }
                "summary.json.enc" => {
                    let decrypted = decrypt_file(&contents, password)?;
                    summary_json = Some(serde_json::from_slice(&decrypted)?);
                }
                "metadata.json.enc" => {
                    let decrypted = decrypt_file(&contents, password)?;
                    metadata_json = Some(serde_json::from_slice(&decrypted)?);
                }
                "terminal_output.log.enc" => {
                    let decrypted = decrypt_file(&contents, password)?;
                    terminal_output = Some(String::from_utf8_lossy(&decrypted).to_string());
                }
                "state_copy.json.enc" => {
                    let decrypted = decrypt_file(&contents, password)?;
                    state_copy_json = Some(serde_json::from_slice(&decrypted)?);
                }
                "integrity.sha256" => {
                    integrity_hash = Some(String::from_utf8_lossy(&contents).trim().to_string());
                }
                _ => {}
            }
        }
        
        Ok(DecryptedData {
            events: events_json.context("Missing events.json.enc")?,
            summary: summary_json.context("Missing summary.json.enc")?,
            metadata: metadata_json.context("Missing metadata.json.enc")?,
            terminal_output: terminal_output.context("Missing terminal_output.log.enc")?,
            state_copy: state_copy_json.context("Missing state_copy.json.enc")?,
            integrity_hash: integrity_hash.context("Missing integrity.sha256")?,
        })
    }
    
    pub fn verify_integrity(&self, password: &str) -> Result<bool> {
        // Read encrypted ZIP
        let encrypted_zip = fs::read(&self.zip_path)
            .context("Failed to read ZIP file")?;
        
        // Decrypt ZIP
        let zip_data = decrypt_file(&encrypted_zip, password)
            .context("Failed to decrypt ZIP file")?;
        
        // Extract files and verify
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_data))
            .context("Failed to open ZIP archive")?;
        
        let mut integrity_hash = None;
        let mut encrypted_files = Vec::new();
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents)?;
            
            let name = file.name().to_string();
            
            if name == "integrity.sha256" {
                integrity_hash = Some(String::from_utf8_lossy(&contents).trim().to_string());
            } else if name.ends_with(".enc") {
                encrypted_files.push((name, contents));
            }
        }
        
        let expected_hash = integrity_hash.context("Missing integrity.sha256")?;
        
        // Calculate hash of all encrypted files
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        for (_, data) in &encrypted_files {
            hasher.update(data);
        }
        let calculated_hash = hex::encode(hasher.finalize());
        
        Ok(calculated_hash == expected_hash)
    }
}

fn decrypt_file(encrypted: &[u8], password: &str) -> Result<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    if encrypted.len() < 12 {
        anyhow::bail!("Invalid encrypted data length");
    }
    
    let key = derive_key_from_password(password)?;
    let cipher = Aes256Gcm::new(&key);
    
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let ciphertext = &encrypted[12..];
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    
    Ok(plaintext)
}

fn derive_key_from_password(password: &str) -> Result<aes_gcm::Key<aes_gcm::Aes256Gcm>> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    
    let salt = b"exam-recorder-suite-salt-v1";
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100000, &mut key);
    
    Ok(*aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(&key))
}

