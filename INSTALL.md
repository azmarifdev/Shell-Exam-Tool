# Installation Guide

## Exam Recorder Suite (ERS)

**Author:** A. Z. M. Arif  
**Website:** [https://azmarif.dev](https://azmarif.dev)

## Prerequisites

- Linux (x86_64/amd64)
- Rust toolchain (for building from source)

## Installation Methods

### Method 1: Debian/Ubuntu Package (.deb)

```bash
# Build the package
./scripts/build-deb.sh

# Install
sudo dpkg -i exam-recorder-suite_1.0.0_amd64.deb

# If dependencies are missing, fix with:
sudo apt-get install -f
```

### Method 2: RPM Package (RedHat/Fedora)

```bash
# Build the package
./scripts/build-rpm.sh

# Install
sudo rpm -i exam-recorder-suite-1.0.0-1.x86_64.rpm
```

### Method 3: Tar Archive

```bash
# Build the archive
./scripts/build-tar.sh

# Extract
tar -xzf exam-recorder-suite-1.0.0-linux-amd64.tar.gz

# Install
cd exam-recorder-suite-1.0.0-linux-amd64
sudo ./install.sh
```

### Method 4: Build from Source

```bash
# Clone repository
git clone <repository-url>
cd exam-recorder-suite

# Build
cargo build --release

# Install manually
sudo cp target/release/exam-recorder /usr/local/bin/
sudo cp target/release/exam-viewer /usr/local/bin/
```

## Verification

After installation, verify the tools are available:

```bash
exam-recorder --version
exam-viewer --version
```

## Configuration

### Instructor Password

**IMPORTANT:** The default instructor password is hardcoded in the exam-recorder binary. For production use, you should:

1. Change the password in `exam-recorder/src/recorder.rs` (line with `instructor_password`)
2. Rebuild the binaries
3. Distribute the updated binaries to students

Alternatively, implement a configuration file or environment variable for the password.

## Usage

See the main [README.md](README.md) for usage instructions.

## Troubleshooting

### Permission Denied

If you get permission errors:

```bash
chmod +x /usr/local/bin/exam-recorder
chmod +x /usr/local/bin/exam-viewer
```

### Missing Dependencies

If binaries fail to run, install required system libraries:

```bash
# Debian/Ubuntu
sudo apt-get install libssl-dev libc6-dev

# RedHat/Fedora
sudo yum install openssl-devel glibc-devel
```

## Security Notes

- The exam-recorder stores encrypted state in `~/.exam-recorder/`
- Logs are stored with restrictive permissions
- Students cannot decrypt or modify logs without the instructor password
- Always verify integrity when analyzing logs

