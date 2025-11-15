use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub username: String,
    pub hostname: String,
    pub machine_id: String,
    pub run_counter: u64,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_seconds: Option<u64>,
}

impl Metadata {
    pub fn new(run_counter: u64) -> Result<Self> {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        
        let machine_id = generate_machine_id(&hostname)?;
        
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(Metadata {
            username,
            hostname,
            machine_id,
            run_counter,
            start_time,
            end_time: None,
            duration_seconds: None,
        })
    }
    
    pub fn finalize(&mut self) {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.end_time = Some(end_time);
        self.duration_seconds = Some(end_time.saturating_sub(self.start_time));
    }
}

fn generate_machine_id(hostname: &str) -> Result<String> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    
    // Try to get MAC address
    if let Ok(output) = Command::new("ip")
        .args(&["link", "show"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.contains("link/ether") {
                    if let Some(mac) = line.split_whitespace().nth(1) {
                        hasher.update(mac.as_bytes());
                        break;
                    }
                }
            }
        }
    }
    
    // Fallback: try /sys/class/net
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(addr) = fs::read_to_string(entry.path().join("address")) {
                let addr = addr.trim();
                if !addr.is_empty() && addr != "00:00:00:00:00:00" {
                    hasher.update(addr.as_bytes());
                    break;
                }
            }
        }
    }
    
    let hash = hasher.finalize();
    Ok(hex::encode(&hash[..16]))
}

