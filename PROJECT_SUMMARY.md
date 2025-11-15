# Exam Recorder Suite - Project Summary

## Overview

**Exam Recorder Suite (ERS)** is a professional-grade system for secure terminal session recording during exams, developed by **A. Z. M. Arif** ([https://azmarif.dev](https://azmarif.dev)).

## Components

### 1. exam-recorder (Student Tool)

**Purpose:** Secure terminal session recorder for students during exams.

**Key Features:**
- Real-time PTY proxy for terminal interaction
- Comprehensive keystroke logging with timestamps
- Paste event detection (bracketed paste mode + timing heuristics)
- Command extraction and logging
- Terminal output recording
- AES-256 encryption for all data
- Password-protected ZIP archive generation
- Encrypted state management with tamper detection
- Run counter tracking

**Output:** Encrypted ZIP file containing:
- `events.json.enc` - All keystroke events
- `summary.json.enc` - Session statistics
- `metadata.json.enc` - User and session metadata
- `terminal_output.log.enc` - Complete terminal output
- `state_copy.json.enc` - State information copy
- `integrity.sha256` - Integrity checksum

### 2. exam-viewer (Instructor Tool)

**Purpose:** Decrypt, analyze, and view student exam logs.

**Key Features:**
- Secure ZIP decryption
- Individual file decryption (AES-256)
- SHA256 integrity verification
- Comprehensive log analysis
- Suspicious activity detection
- Command timeline extraction
- Typing statistics
- Report generation (text, markdown, JSON)
- Colorized output for readability

**Commands:**
- `open` - Full analysis and report
- `summary` - Quick summary only
- `verify` - Integrity check only
- `export` - Export to file (--pdf, --markdown, --json)

## Security Architecture

### Encryption
- **Algorithm:** AES-256-GCM
- **Key Derivation:** PBKDF2-HMAC-SHA256 (100,000 iterations)
- **Salt:** Hardcoded application salt

### Integrity
- **Hash Algorithm:** SHA256
- **Verification:** File-level and archive-level checksums
- **Tamper Detection:** Multiple integrity checkpoints

### State Protection
- Encrypted state file (`~/.exam-recorder/state.json.enc`)
- Machine-specific key derivation
- Checksummed data
- Restrictive file permissions (000)

### Log Protection
- Encrypted individual files
- Password-protected ZIP archive
- Restrictive permissions (600)
- Cannot be decrypted without instructor password

## File Structure

```
exam-recorder-suite/
├── Cargo.toml              # Workspace configuration
├── README.md                # Main documentation
├── INSTALL.md               # Installation guide
├── CONFIGURATION.md         # Configuration guide
├── LICENSE                  # MIT License
├── build.sh                 # Quick build script
├── scripts/                  # Distribution scripts
│   ├── build-deb.sh        # Debian package builder
│   ├── build-rpm.sh        # RPM package builder
│   └── build-tar.sh        # Tar archive builder
├── exam-recorder/           # Student tool
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Entry point
│       ├── recorder.rs     # Main recording logic
│       ├── encryption.rs   # Encryption utilities
│       ├── state.rs        # State management
│       └── metadata.rs     # Metadata collection
└── exam-viewer/            # Instructor tool
    ├── Cargo.toml
    └── src/
        ├── main.rs         # Entry point
        ├── decryptor.rs    # Decryption logic
        ├── analyzer.rs     # Log analysis
        └── reporter.rs     # Report generation
```

## Technology Stack

- **Language:** Rust (Edition 2021)
- **Key Dependencies:**
  - `aes-gcm` - AES-256-GCM encryption
  - `nix` - Unix system calls (PTY, fork, etc.)
  - `termios` - Terminal control
  - `zip` - ZIP archive creation
  - `sha2` - SHA256 hashing
  - `serde` - Serialization
  - `clap` - CLI argument parsing
  - `rpassword` - Secure password input
  - `colored` - Terminal colors

## Build & Distribution

### Build from Source
```bash
cargo build --release
```

### Distribution Packages
- **Debian/Ubuntu:** `.deb` package via `build-deb.sh`
- **RedHat/Fedora:** `.rpm` package via `build-rpm.sh`
- **Generic Linux:** `.tar.gz` archive via `build-tar.sh`

## Usage Workflow

### Student Side
1. Student runs `exam-recorder`
2. Tool starts recording terminal session
3. Student completes exam work
4. Student types `exit` to finish
5. Tool generates encrypted ZIP file
6. Student submits ZIP to instructor

### Instructor Side
1. Instructor receives ZIP file from student
2. Instructor runs `exam-viewer open <file.zip>`
3. Tool prompts for decryption password
4. Tool decrypts and verifies integrity
5. Tool generates comprehensive report
6. Instructor reviews analysis and suspicious activities

## Security Considerations

### Before Production Use
1. **Change instructor password** in `exam-recorder/src/recorder.rs`
2. **Rebuild binaries** with new password
3. **Test decryption** with exam-viewer
4. **Distribute securely** to students
5. **Store password securely** (password manager)

### Best Practices
- Use strong passwords (16+ characters)
- Verify integrity of distributed binaries
- Use secure channels for log submission
- Regularly audit exam logs
- Keep instructor password confidential

## Limitations & Future Enhancements

### Current Limitations
- Instructor password is hardcoded (must rebuild to change)
- No configuration file support
- PDF export uses text format (not true PDF)
- Limited to Linux platforms
- No network-based submission

### Planned Enhancements
- Configuration file support
- Environment variable for password
- True PDF generation
- Windows/macOS support
- Network submission with encryption
- Web-based viewer interface
- Advanced analytics and visualizations

## Testing

To test the system:

1. Build both tools:
   ```bash
   cargo build --release
   ```

2. Run exam-recorder:
   ```bash
   ./target/release/exam-recorder
   ```

3. Type some commands, then type `exit`

4. Decrypt and view:
   ```bash
   ./target/release/exam-viewer open ~/.exam-recorder/exam-result-*.zip
   ```

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Author

**A. Z. M. Arif**  
Website: [https://azmarif.dev](https://azmarif.dev)

