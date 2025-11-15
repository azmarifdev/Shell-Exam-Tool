# Configuration Guide

## Exam Recorder Suite Configuration

### Instructor Password

The exam-recorder tool uses a hardcoded instructor password to encrypt exam logs. **This password must be changed before production use.**

#### Changing the Password

1. Open `exam-recorder/src/recorder.rs`
2. Find the line with `instructor_password` (around line 390)
3. Change the password to a secure value
4. Rebuild the binaries:
   ```bash
   cargo build --release
   ```

#### Password Requirements

- Use a strong password (minimum 16 characters)
- Include uppercase, lowercase, numbers, and special characters
- Do not share this password with students
- Store it securely (password manager recommended)

### State Storage

The exam-recorder stores encrypted state in:
```
~/.exam-recorder/state.json.enc
```

This file:
- Is encrypted with machine-specific keys
- Contains run counter and last run time
- Has restrictive permissions (000)
- Cannot be modified by students

### Log Storage

Exam logs are stored in:
```
~/.exam-recorder/exam-result-<username>-<timestamp>.zip
```

These files:
- Are password-protected ZIP archives
- Contain encrypted JSON files
- Include integrity checksums
- Have restrictive permissions (600)

### Environment Variables

Currently, the tools use the following environment variables:

- `USER` or `USERNAME` - For identifying the student
- `SHELL` - For determining which shell to execute (defaults to `/bin/bash`)
- `TERM` - Set to `xterm-256color` for proper terminal emulation

### Future Configuration Options

Planned configuration features:

- Configuration file support (`~/.exam-recorder/config.toml`)
- Environment variable for instructor password
- Custom log storage location
- Adjustable paste detection thresholds
- Customizable suspicious activity detection rules

## Security Considerations

1. **Password Management**: Never hardcode passwords in production. Use secure configuration management.

2. **State Protection**: The state file is encrypted, but physical access to the machine could allow tampering. Consider additional protections for high-security environments.

3. **Network Security**: The tools operate offline by default. If logs are transferred over a network, use secure channels (SSH, encrypted email, etc.).

4. **Audit Trail**: Consider logging when exam-recorder is executed for additional security auditing.

5. **Binary Integrity**: Distribute binaries through secure channels and verify checksums before installation.

