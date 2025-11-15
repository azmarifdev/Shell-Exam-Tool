use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::encryption::{encrypt_file, calculate_file_hash, create_password_protected_zip};
use crate::metadata::Metadata;
use crate::state::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub timestamp: u64,
    pub key_code: u32,
    pub key_name: String,
    pub raw_bytes: Vec<u8>,
    pub is_paste: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    pub timestamp: u64,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_keystrokes: usize,
    pub enter_pressed: usize,
    pub backspace_used: usize,
    pub delete_used: usize,
    pub paste_events: usize,
    pub total_pasted_chars: usize,
    pub commands_executed: usize,
}

pub struct Recorder {
    state: State,
    metadata: Metadata,
    keystrokes: Vec<KeystrokeEvent>,
    commands: Vec<CommandEvent>,
    terminal_output: Vec<u8>,
    current_input: String,
    paste_detector: PasteDetector,
}

struct PasteDetector {
    recent_keystrokes: VecDeque<(Instant, usize)>,
    threshold_chars_per_ms: usize,
    min_chars_for_paste: usize,
}

impl PasteDetector {
    fn new() -> Self {
        Self {
            recent_keystrokes: VecDeque::new(),
            threshold_chars_per_ms: 10, // 10 chars per millisecond = paste
            min_chars_for_paste: 20,
        }
    }
    
    fn check_paste(&mut self, char_count: usize) -> bool {
        let now = Instant::now();
        self.recent_keystrokes.push_back((now, char_count));
        
        // Keep only last 100ms
        while let Some(&(time, _)) = self.recent_keystrokes.front() {
            if now.duration_since(time) > Duration::from_millis(100) {
                self.recent_keystrokes.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we have enough characters in short time
        let total_chars: usize = self.recent_keystrokes.iter().map(|(_, c)| c).sum();
        let time_span = if let (Some(first), Some(last)) = 
            (self.recent_keystrokes.front(), self.recent_keystrokes.back()) {
            last.0.duration_since(first.0).as_millis().max(1) as usize
        } else {
            return false;
        };
        
        if total_chars >= self.min_chars_for_paste {
            let chars_per_ms = total_chars * 1000 / time_span;
            if chars_per_ms >= self.threshold_chars_per_ms {
                self.recent_keystrokes.clear();
                return true;
            }
        }
        
        false
    }
}

impl Recorder {
    pub fn new() -> Result<Self> {
        let mut state = State::load()?;
        state.increment_counter();
        state.save()?;
        
        let metadata = Metadata::new(state.run_counter)?;
        
        Ok(Recorder {
            state,
            metadata,
            keystrokes: Vec::new(),
            commands: Vec::new(),
            terminal_output: Vec::new(),
            current_input: String::new(),
            paste_detector: PasteDetector::new(),
        })
    }
    
    pub fn start(&mut self) -> Result<()> {
        use std::os::unix::io::{AsRawFd, RawFd};
        use nix::pty::{openpty, Pty};
        use nix::unistd::{fork, ForkResult, execvp, dup2, close};
        use nix::sys::wait::waitpid;
        use std::ffi::CString;
        
        // Create PTY
        let pty_pair = openpty(None, None)
            .context("Failed to create PTY")?;
        
        let master_fd = pty_pair.master.as_raw_fd();
        let slave_fd = pty_pair.slave.as_raw_fd();
        
        // Fork process
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                // Parent: record from master
                close(slave_fd).ok();
                self.record_from_master(master_fd)?;
                waitpid(child, None).ok();
            }
            Ok(ForkResult::Child) => {
                // Child: execute shell
                close(master_fd).ok();
                
                // Connect slave to stdin/stdout/stderr
                dup2(slave_fd, 0).ok();
                dup2(slave_fd, 1).ok();
                dup2(slave_fd, 2).ok();
                close(slave_fd).ok();
                
                // Set TERM environment
                std::env::set_var("TERM", "xterm-256color");
                
                // Execute shell
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
                let shell_cstr = CString::new(shell.clone())
                    .context("Invalid shell path")?;
                let args = vec![shell_cstr.clone()];
                
                execvp(&shell_cstr, &args)
                    .expect("Failed to execute shell");
            }
            Err(e) => {
                anyhow::bail!("Fork failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn record_from_master(&mut self, master_fd: RawFd) -> Result<()> {
        use std::os::unix::io::RawFd;
        use std::os::unix::io::FromRawFd;
        
        let mut master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
        let mut stdin = std::io::stdin();
        
        // Set terminal to raw mode
        let original_termios = self.set_raw_mode()?;
        
        // Use poll/epoll for non-blocking I/O
        use nix::poll::{poll, PollFd, PollFlags};
        
        let result = loop {
            let mut poll_fds = vec![
                PollFd::new(0, PollFlags::POLLIN), // stdin
                PollFd::new(master_fd, PollFlags::POLLIN), // master PTY
            ];
            
            match poll(&mut poll_fds, 100) {
                Ok(0) => continue, // Timeout
                Ok(_) => {}
                Err(nix::errno::Errno::EINTR) => continue,
                Err(e) => break Err(e.into()),
            }
            
            // Read from stdin (user input)
            if let Some(revents) = poll_fds[0].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    let mut buffer = [0u8; 4096];
                    match stdin.read(&mut buffer) {
                        Ok(0) => break Ok(()), // EOF
                        Ok(n) => {
                            let data = &buffer[..n];
                            self.process_input(data)?;
                            
                            // Forward to master PTY
                            master_file.write_all(data)?;
                            master_file.flush()?;
                            
                            // Check for exit command after processing
                            if self.should_exit() {
                                break Ok(());
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => break Err(e.into()),
                    }
                }
            }
            
            // Read from master PTY (terminal output)
            if let Some(revents) = poll_fds[1].revents() {
                if revents.contains(PollFlags::POLLIN) {
                    let mut buffer = [0u8; 4096];
                    match master_file.read(&mut buffer) {
                        Ok(0) => break Ok(()), // EOF
                        Ok(n) => {
                            let data = &buffer[..n];
                            self.terminal_output.extend_from_slice(data);
                            
                            // Forward to stdout
                            std::io::stdout().write_all(data)?;
                            std::io::stdout().flush()?;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => break Err(e.into()),
                    }
                }
            }
        };
        
        // Restore terminal mode (always, even on error)
        let _ = self.restore_terminal_mode(original_termios);
        
        result?;
        
        // Generate output files
        self.finalize()?;
        
        Ok(())
    }
    
    fn set_raw_mode(&self) -> Result<termios::Termios> {
        use termios::*;
        
        let stdin_fd = 0;
        let mut termios = Termios::from_fd(stdin_fd)?;
        let original = termios.clone();
        
        // Set raw mode
        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios.c_oflag &= !OPOST;
        termios.c_cflag |= CS8;
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;
        
        tcsetattr(stdin_fd, TCSANOW, &termios)?;
        
        Ok(original)
    }
    
    fn restore_terminal_mode(&self, termios: termios::Termios) -> Result<()> {
        use termios::*;
        tcsetattr(0, TCSANOW, &termios)?;
        Ok(())
    }
    
    fn process_input(&mut self, data: &[u8]) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        for &byte in data {
            let (key_name, is_special) = self.decode_key(byte);
            let is_paste = if !is_special && byte >= 32 && byte < 127 {
                // Check for paste detection
                self.paste_detector.check_paste(1)
            } else {
                false
            };
            
            // Track current input for command detection
            if byte == b'\n' || byte == b'\r' {
                if !self.current_input.trim().is_empty() {
                    let cmd = self.current_input.trim().to_string();
                    // Don't record "exit" as a command
                    if cmd != "exit" {
                        self.commands.push(CommandEvent {
                            timestamp,
                            command: cmd,
                        });
                    }
                }
                self.current_input.clear();
            } else if byte == 127 || byte == 8 { // Backspace
                self.current_input.pop();
            } else if byte >= 32 && byte < 127 && !is_special {
                self.current_input.push(byte as char);
            }
            
            self.keystrokes.push(KeystrokeEvent {
                timestamp,
                key_code: byte as u32,
                key_name,
                raw_bytes: vec![byte],
                is_paste,
            });
        }
        
        Ok(())
    }
    
    pub fn should_exit(&self) -> bool {
        self.current_input.trim() == "exit"
    }
    
    fn decode_key(&self, byte: u8) -> (String, bool) {
        match byte {
            0 => ("NULL".to_string(), true),
            1 => ("Ctrl+A".to_string(), true),
            2 => ("Ctrl+B".to_string(), true),
            3 => ("Ctrl+C".to_string(), true),
            4 => ("Ctrl+D".to_string(), true),
            5 => ("Ctrl+E".to_string(), true),
            6 => ("Ctrl+F".to_string(), true),
            7 => ("Ctrl+G".to_string(), true),
            8 | 127 => ("Backspace".to_string(), true),
            9 => ("Tab".to_string(), true),
            10 | 13 => ("Enter".to_string(), true),
            27 => ("ESC".to_string(), true),
            32..=126 => (format!("{}", byte as char), false),
            _ => (format!("0x{:02X}", byte), true),
        }
    }
    
    fn finalize(&mut self) -> Result<()> {
        self.metadata.finalize();
        
        // Generate summary
        let summary = self.generate_summary();
        
        // Create output directory
        let output_dir = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".exam-recorder");
        std::fs::create_dir_all(&output_dir)?;
        
        // Generate filename
        let timestamp = self.metadata.start_time;
        let filename = format!(
            "exam-result-{}-{}.zip",
            self.metadata.username,
            timestamp
        );
        let output_path = output_dir.join(&filename);
        
        // Serialize data
        let events_json = serde_json::to_string_pretty(&self.keystrokes)?;
        let summary_json = serde_json::to_string_pretty(&summary)?;
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        let state_copy_json = serde_json::to_string_pretty(&self.state)?;
        
        // IMPORTANT: Change this password before production use!
        // See CONFIGURATION.md for instructions on changing the instructor password.
        // This password is used to encrypt all exam log files.
        let instructor_password = "instructor_password_change_me";
        
        let events_enc = encrypt_file(events_json.as_bytes(), instructor_password)?;
        let summary_enc = encrypt_file(summary_json.as_bytes(), instructor_password)?;
        let metadata_enc = encrypt_file(metadata_json.as_bytes(), instructor_password)?;
        let terminal_output_enc = encrypt_file(&self.terminal_output, instructor_password)?;
        let state_copy_enc = encrypt_file(state_copy_json.as_bytes(), instructor_password)?;
        
        // Calculate integrity hash
        let mut integrity_data = Vec::new();
        integrity_data.extend_from_slice(&events_enc);
        integrity_data.extend_from_slice(&summary_enc);
        integrity_data.extend_from_slice(&metadata_enc);
        integrity_data.extend_from_slice(&terminal_output_enc);
        integrity_data.extend_from_slice(&state_copy_enc);
        let integrity_hash = calculate_file_hash(&integrity_data);
        
        // Create ZIP with password protection
        let zip_files = vec![
            ("events.json.enc", events_enc),
            ("summary.json.enc", summary_enc),
            ("metadata.json.enc", metadata_enc),
            ("terminal_output.log.enc", terminal_output_enc),
            ("state_copy.json.enc", state_copy_enc),
            ("integrity.sha256", integrity_hash.as_bytes().to_vec()),
        ];
        
        let encrypted_zip = create_password_protected_zip(&zip_files, instructor_password)?;
        
        // Write ZIP file
        std::fs::write(&output_path, encrypted_zip)?;
        
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&output_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&output_path, perms)?;
        }
        
        println!();
        println!("Session ended.");
        println!("Your encrypted exam log has been saved as: {}", filename);
        println!("Submit this file to your instructor.");
        
        Ok(())
    }
    
    fn generate_summary(&self) -> SessionSummary {
        let mut summary = SessionSummary {
            total_keystrokes: self.keystrokes.len(),
            enter_pressed: 0,
            backspace_used: 0,
            delete_used: 0,
            paste_events: 0,
            total_pasted_chars: 0,
            commands_executed: self.commands.len(),
        };
        
        for keystroke in &self.keystrokes {
            match keystroke.key_name.as_str() {
                "Enter" => summary.enter_pressed += 1,
                "Backspace" => summary.backspace_used += 1,
                "Delete" => summary.delete_used += 1,
                _ => {}
            }
            
            if keystroke.is_paste {
                summary.paste_events += 1;
                summary.total_pasted_chars += keystroke.raw_bytes.len();
            }
        }
        
        summary
    }
}

