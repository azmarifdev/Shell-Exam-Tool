use anyhow::Result;
use std::fs;
use std::path::Path;
use colored::*;

use crate::analyzer::AnalysisReport;

pub struct Reporter;

impl Reporter {
    pub fn new() -> Self {
        Reporter
    }
    
    pub fn print_full_report(&self, report: &AnalysisReport) -> Result<()> {
        println!("=== Exam Viewer Report ===");
        println!("Author: A. Z. M. Arif | https://azmarif.dev");
        println!();
        println!("Student Username:        {}", report.username);
        println!("Hostname:                {}", report.hostname);
        println!("Machine ID:              {}", report.machine_id);
        println!("Session Duration:        {}", report.session_duration);
        println!("Recorder Runs Before:    {}", report.recorder_runs_before);
        println!();
        println!("--- Typing Statistics ---");
        println!("Total Keystrokes:        {}", report.total_keystrokes);
        println!("Enter Pressed:           {}", report.enter_pressed);
        println!("Backspace Used:          {}", report.backspace_used);
        println!("Paste Events:            {}", report.paste_events);
        println!("Total Pasted Characters: {}", report.total_pasted_chars);
        println!();
        
        if !report.commands.is_empty() {
            println!("--- Command Timeline ---");
            for (i, cmd) in report.commands.iter().enumerate() {
                println!("{}. {}", i + 1, cmd);
            }
            println!();
        }
        
        if !report.suspicious_activities.is_empty() {
            println!("--- Suspicious Activity ---");
            for activity in &report.suspicious_activities {
                let severity_color = match activity.severity.as_str() {
                    "HIGH" => activity.description.red().bold(),
                    "MEDIUM" => activity.description.yellow(),
                    _ => activity.description.normal(),
                };
                println!("[!] {} at {}", severity_color, activity.timestamp);
            }
            println!();
        }
        
        println!("--- Integrity ---");
        if report.integrity_passed {
            println!("SHA256 check: {}", "PASSED".green().bold());
        } else {
            println!("SHA256 check: {}", "FAILED - TAMPERED".red().bold());
        }
        
        Ok(())
    }
    
    pub fn print_summary(&self, report: &AnalysisReport) -> Result<()> {
        println!("=== Exam Summary ===");
        println!("Student: {}", report.username);
        println!("Duration: {}", report.session_duration);
        println!("Keystrokes: {}", report.total_keystrokes);
        println!("Paste Events: {}", report.paste_events);
        println!("Commands: {}", report.commands.len());
        println!("Integrity: {}", 
            if report.integrity_passed { "PASSED" } else { "FAILED" });
        Ok(())
    }
    
    pub fn export_pdf(&self, report: &AnalysisReport, path: &Path) -> Result<()> {
        // For now, export as text - PDF generation would require additional dependencies
        // In production, use a PDF library like printpdf
        let content = self.generate_text_report(report);
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn export_markdown(&self, report: &AnalysisReport, path: &Path) -> Result<()> {
        let mut content = String::new();
        content.push_str("# Exam Viewer Report\n\n");
        content.push_str("**Author:** A. Z. M. Arif | https://azmarif.dev\n\n");
        content.push_str(&format!("**Student Username:** {}\n", report.username));
        content.push_str(&format!("**Hostname:** {}\n", report.hostname));
        content.push_str(&format!("**Machine ID:** {}\n", report.machine_id));
        content.push_str(&format!("**Session Duration:** {}\n", report.session_duration));
        content.push_str(&format!("**Recorder Runs Before:** {}\n\n", report.recorder_runs_before));
        
        content.push_str("## Typing Statistics\n\n");
        content.push_str(&format!("- Total Keystrokes: {}\n", report.total_keystrokes));
        content.push_str(&format!("- Enter Pressed: {}\n", report.enter_pressed));
        content.push_str(&format!("- Backspace Used: {}\n", report.backspace_used));
        content.push_str(&format!("- Paste Events: {}\n", report.paste_events));
        content.push_str(&format!("- Total Pasted Characters: {}\n\n", report.total_pasted_chars));
        
        if !report.commands.is_empty() {
            content.push_str("## Command Timeline\n\n");
            for (i, cmd) in report.commands.iter().enumerate() {
                content.push_str(&format!("{}. `{}`\n", i + 1, cmd));
            }
            content.push_str("\n");
        }
        
        if !report.suspicious_activities.is_empty() {
            content.push_str("## Suspicious Activity\n\n");
            for activity in &report.suspicious_activities {
                content.push_str(&format!("- **{}** at {}: {}\n", 
                    activity.severity, activity.timestamp, activity.description));
            }
            content.push_str("\n");
        }
        
        content.push_str("## Integrity\n\n");
        content.push_str(&format!("SHA256 check: {}\n", 
            if report.integrity_passed { "PASSED" } else { "FAILED - TAMPERED" }));
        
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn export_json(&self, report: &AnalysisReport, path: &Path) -> Result<()> {
        let json = serde_json::json!({
            "username": report.username,
            "hostname": report.hostname,
            "machine_id": report.machine_id,
            "session_duration": report.session_duration,
            "recorder_runs_before": report.recorder_runs_before,
            "total_keystrokes": report.total_keystrokes,
            "enter_pressed": report.enter_pressed,
            "backspace_used": report.backspace_used,
            "paste_events": report.paste_events,
            "total_pasted_chars": report.total_pasted_chars,
            "commands": report.commands,
            "suspicious_activities": report.suspicious_activities.iter().map(|a| {
                serde_json::json!({
                    "timestamp": a.timestamp,
                    "description": a.description,
                    "severity": a.severity,
                })
            }).collect::<Vec<_>>(),
            "integrity_passed": report.integrity_passed,
        });
        
        let content = serde_json::to_string_pretty(&json)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn generate_text_report(&self, report: &AnalysisReport) -> String {
        let mut content = String::new();
        content.push_str("=== Exam Viewer Report ===\n");
        content.push_str("Author: A. Z. M. Arif | https://azmarif.dev\n\n");
        content.push_str(&format!("Student Username:        {}\n", report.username));
        content.push_str(&format!("Hostname:                {}\n", report.hostname));
        content.push_str(&format!("Machine ID:              {}\n", report.machine_id));
        content.push_str(&format!("Session Duration:        {}\n", report.session_duration));
        content.push_str(&format!("Recorder Runs Before:    {}\n\n", report.recorder_runs_before));
        content.push_str("--- Typing Statistics ---\n");
        content.push_str(&format!("Total Keystrokes:        {}\n", report.total_keystrokes));
        content.push_str(&format!("Enter Pressed:           {}\n", report.enter_pressed));
        content.push_str(&format!("Backspace Used:          {}\n", report.backspace_used));
        content.push_str(&format!("Paste Events:            {}\n", report.paste_events));
        content.push_str(&format!("Total Pasted Characters: {}\n\n", report.total_pasted_chars));
        
        if !report.commands.is_empty() {
            content.push_str("--- Command Timeline ---\n");
            for (i, cmd) in report.commands.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, cmd));
            }
            content.push_str("\n");
        }
        
        if !report.suspicious_activities.is_empty() {
            content.push_str("--- Suspicious Activity ---\n");
            for activity in &report.suspicious_activities {
                content.push_str(&format!("[!] {} at {}: {}\n", 
                    activity.severity, activity.timestamp, activity.description));
            }
            content.push_str("\n");
        }
        
        content.push_str("--- Integrity ---\n");
        content.push_str(&format!("SHA256 check: {}\n", 
            if report.integrity_passed { "PASSED" } else { "FAILED - TAMPERED" }));
        
        content
    }
}

