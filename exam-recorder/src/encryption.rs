use anyhow::{Context, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Sha256, Digest};

pub fn encrypt_file(data: &[u8], password: &str) -> Result<Vec<u8>> {
    let key = derive_key_from_password(password)?;
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

pub fn decrypt_file(encrypted: &[u8], password: &str) -> Result<Vec<u8>> {
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

pub fn calculate_file_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn create_password_protected_zip(
    files: &[(&str, Vec<u8>)],
    password: &str,
) -> Result<Vec<u8>> {
    use std::io::Write;
    
    let mut zip_data = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_data));
        
        // Note: The zip crate doesn't support password-protected encryption directly
        // We'll encrypt the entire ZIP file after creation
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(9));
        
        for (filename, data) in files {
            zip.start_file(*filename, options)
                .context("Failed to start zip file entry")?;
            zip.write_all(data)
                .context("Failed to write zip file data")?;
        }
        
        zip.finish()
            .context("Failed to finish zip file")?;
    }
    
    // Encrypt the entire ZIP with AES-256
    // The password protection is handled by encrypting the ZIP itself
    encrypt_file(&zip_data, password)
}

