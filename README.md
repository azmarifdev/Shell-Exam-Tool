# Exam Recorder Suite (ERS)

**Author:** A. Z. M. Arif  
**Website:** [https://azmarif.dev](https://azmarif.dev)

A professional-grade system for secure terminal session recording during exams, consisting of two components:

1. **exam-recorder** - Student-side secure terminal session recorder
2. **exam-viewer** - Instructor-side decrypter, analyzer, and log viewer

## Features

### exam-recorder
- Real-time terminal activity logging (keystrokes, commands, output)
- Paste event detection (bracketed paste mode + timing heuristics)
- AES-256 encryption with password-protected ZIP output
- Tamper-resistant with integrity checks
- Hidden state management with encrypted storage
- Cross-platform support (Linux-focused)

### exam-viewer
- Secure decryption of student exam logs
- Integrity verification (SHA256)
- Comprehensive analysis and reporting
- Suspicious activity detection
- Export to PDF/Markdown/JSON
- Timeline replay support

## Installation

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

### Quick Start (From Source)

```bash
git clone <repository-url>
cd exam-recorder-suite
cargo build --release
sudo cp target/release/exam-recorder /usr/local/bin/
sudo cp target/release/exam-viewer /usr/local/bin/
```

### Distribution Packages

Build scripts are provided for:
- `.deb` (Debian/Ubuntu) - `./scripts/build-deb.sh`
- `.rpm` (RedHat/Fedora) - `./scripts/build-rpm.sh`
- `.tar.gz` (Generic Linux) - `./scripts/build-tar.sh`

See [INSTALL.md](INSTALL.md) for details.

## Usage

### Student Side (exam-recorder)

```bash
exam-recorder
```

The tool will:
- Start recording terminal activity
- Log all keystrokes, commands, and output
- Generate encrypted ZIP on exit
- Save to: `exam-result-<username>-<timestamp>.zip`

### Instructor Side (exam-viewer)

```bash
# Open and analyze exam log
exam-viewer open exam-result-username-12345.zip

# Get summary only
exam-viewer summary exam-result-username-12345.zip

# Verify integrity
exam-viewer verify exam-result-username-12345.zip

# Export report
exam-viewer export exam-result-username-12345.zip --pdf report.pdf
```

## Security

- All data encrypted with AES-256
- Password-protected ZIP archives
- SHA256 integrity verification
- Tamper detection mechanisms
- Encrypted state storage
- Restricted file permissions

**Important:** Before production use, change the instructor password in `exam-recorder/src/recorder.rs` and rebuild. See [CONFIGURATION.md](CONFIGURATION.md) for details.

## License

MIT License

## Documentation

- [Installation Guide](INSTALL.md) - Detailed installation instructions
- [Configuration Guide](CONFIGURATION.md) - Configuration and security settings
- [LICENSE](LICENSE) - MIT License

## Architecture

```
       ┌──────────────────────────────────────────┐
       │        Student Computer (Offline)        │
       ├──────────────────────────────────────────┤
       │ exam-recorder                            │
       │  - PTY Proxy                             │
       │  - Keystroke Logger                      │
       │  - Paste Detector                        │
       │  - Output Recorder                       │
       │  - Metadata Collector                    │
       │  - State Manager (enc + protected)       │
       │  - AES-256 Encryptor                     │
       │  - ZIP Generator (AES-256)              │
       └──────────────┬───────────────────────────┘
                      │
                      │ Encrypted ZIP (submitted manually)
                      ▼
       ┌──────────────────────────────────────────┐
       │          Instructor Computer              │
       ├──────────────────────────────────────────┤
       │ exam-viewer                              │
       │  - ZIP Decryptor                         │
       │  - AES-256 File Decryptor                │
       │  - Integrity Verifier (SHA256)           │
       │  - Log Parser                            │
       │  - Analyzer + Suspicion Detector         │
       │  - Report Generator (JSON/TXT/PDF)       │
       └──────────────────────────────────────────┘
```

## Author

A. Z. M. Arif - [https://azmarif.dev](https://azmarif.dev)

