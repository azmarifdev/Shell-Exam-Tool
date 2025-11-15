use anyhow::Result;
use serde_json::Value;
use chrono::{DateTime, Utc, NaiveDateTime};

pub struct DecryptedData {
    pub events: Value,
    pub summary: Value,
    pub metadata: Value,
    pub terminal_output: String,
    pub state_copy: Value,
    pub integrity_hash: String,
}

pub struct AnalysisReport {
    pub username: String,
    pub hostname: String,
    pub machine_id: String,
    pub session_duration: String,
    pub recorder_runs_before: u64,
    pub total_keystrokes: usize,
    pub enter_pressed: usize,
    pub backspace_used: usize,
    pub paste_events: usize,
    pub total_pasted_chars: usize,
    pub commands: Vec<String>,
    pub suspicious_activities: Vec<SuspiciousActivity>,
    pub integrity_passed: bool,
}

pub struct SuspiciousActivity {
    pub timestamp: String,
    pub description: String,
    pub severity: String,
}

pub struct Analyzer {
    data: DecryptedData,
}

impl Analyzer {
    pub fn new(data: DecryptedData) -> Self {
        Analyzer { data }
    }
    
    pub fn analyze(&self) -> Result<AnalysisReport> {
        let metadata = &self.data.metadata;
        let summary = &self.data.summary;
        let events = &self.data.events;
        
        let username = metadata["username"].as_str()
            .unwrap_or("unknown").to_string();
        let hostname = metadata["hostname"].as_str()
            .unwrap_or("unknown").to_string();
        let machine_id = metadata["machine_id"].as_str()
            .unwrap_or("unknown").to_string();
        let recorder_runs_before = metadata["run_counter"].as_u64()
            .unwrap_or(0)
            .saturating_sub(1);
        
        let start_time = metadata["start_time"].as_u64().unwrap_or(0);
        let end_time = metadata["end_time"].as_u64().unwrap_or(start_time);
        let duration_secs = end_time.saturating_sub(start_time);
        let session_duration = format_duration(duration_secs);
        
        let total_keystrokes = summary["total_keystrokes"].as_u64()
            .unwrap_or(0) as usize;
        let enter_pressed = summary["enter_pressed"].as_u64()
            .unwrap_or(0) as usize;
        let backspace_used = summary["backspace_used"].as_u64()
            .unwrap_or(0) as usize;
        let paste_events = summary["paste_events"].as_u64()
            .unwrap_or(0) as usize;
        let total_pasted_chars = summary["total_pasted_chars"].as_u64()
            .unwrap_or(0) as usize;
        
        // Extract commands
        let mut commands = Vec::new();
        if let Some(events_array) = events.as_array() {
            let mut current_command = String::new();
            for event in events_array {
                if let Some(key_name) = event["key_name"].as_str() {
                    if key_name == "Enter" {
                        if !current_command.trim().is_empty() {
                            commands.push(current_command.trim().to_string());
                            current_command.clear();
                        }
                    } else if let Some(raw_bytes) = event["raw_bytes"].as_array() {
                        for byte_val in raw_bytes {
                            if let Some(byte) = byte_val.as_u64() {
                                let b = byte as u8;
                                if b >= 32 && b < 127 && b != 127 {
                                    current_command.push(b as char);
                                } else if b == 127 || b == 8 {
                                    current_command.pop();
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Detect suspicious activities
        let suspicious_activities = self.detect_suspicious_activities(events)?;
        
        // Verify integrity
        let integrity_passed = self.verify_integrity()?;
        
        Ok(AnalysisReport {
            username,
            hostname,
            machine_id,
            session_duration,
            recorder_runs_before,
            total_keystrokes,
            enter_pressed,
            backspace_used,
            paste_events,
            total_pasted_chars,
            commands,
            suspicious_activities,
            integrity_passed,
        })
    }
    
    fn detect_suspicious_activities(&self, events: &Value) -> Result<Vec<SuspiciousActivity>> {
        let mut activities = Vec::new();
        
        if let Some(events_array) = events.as_array() {
            for event in events_array {
                if let Some(is_paste) = event["is_paste"].as_bool() {
                    if is_paste {
                        let timestamp = event["timestamp"].as_u64().unwrap_or(0);
                        let timestamp_str = format_timestamp(timestamp);
                        let raw_bytes = event["raw_bytes"].as_array()
                            .map(|arr| arr.len())
                            .unwrap_or(0);
                        
                        activities.push(SuspiciousActivity {
                            timestamp: timestamp_str,
                            description: format!(
                                "Detected paste burst ({} chars)",
                                raw_bytes
                            ),
                            severity: if raw_bytes > 100 { "HIGH".to_string() } else { "MEDIUM".to_string() },
                        });
                    }
                }
            }
        }
        
        Ok(activities)
    }
    
    fn verify_integrity(&self) -> Result<bool> {
        use sha2::{Sha256, Digest};
        
        // Recalculate hash from decrypted data
        let mut hasher = Sha256::new();
        hasher.update(self.data.events.to_string().as_bytes());
        hasher.update(self.data.summary.to_string().as_bytes());
        hasher.update(self.data.metadata.to_string().as_bytes());
        hasher.update(self.data.terminal_output.as_bytes());
        hasher.update(self.data.state_copy.to_string().as_bytes());
        
        let calculated = hex::encode(hasher.finalize());
        Ok(calculated == self.data.integrity_hash)
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else {
        format!("{}m {:02}s", minutes, secs)
    }
}

fn format_timestamp(timestamp_ms: u64) -> String {
    let timestamp_secs = timestamp_ms / 1000;
    if let Some(dt) = NaiveDateTime::from_timestamp_opt(timestamp_secs as i64, 0) {
        dt.format("%H:%M:%S").to_string()
    } else {
        format!("{}", timestamp_ms)
    }
}

