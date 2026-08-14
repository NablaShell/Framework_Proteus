# Security Policy

## Supported Versions

| Version | Codename | Supported          |
| ------- | -------- | ------------------ |
| 1.0.0   | Fulcrum  | :white_check_mark: |

## Reporting a Vulnerability

**Do NOT open public issues for security vulnerabilities.**

### Contact
- **Email:** nablashell@gmail.com
- **Response Time:** 24-48 hours

### PGP Key

**Fingerprint:** `3CD4 9510 BA6C 43FD 44ED  F85E DFA3 E21A F7A6 3A8C`

**Public Key:**

```
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEan7tKBYJKwYBBAHaRw8BAQdANyARUGs3AL6NovzUwZ5927Z/fCfqXIJW/1Wn
W07XkUm0L1Byb2plY3QgUHJvdGV1cyBTZWN1cml0eSA8bmFibGFzaGVsbEBnbWFp
bC5jb20+iJYEExYKAD4WIQQ81JUQumxD/UTt+F7fo+Ia96Y6jAUCan7tKAIbAwUJ
AeEzgAULCQgHAgYVCgkICwIEFgIDAQIeAQIXgAAKCRDfo+Ia96Y6jFqDAP9GWGY1
biv4eWWgSoVXzdmxK7bUiOMDRWrbGX8JN1hRbgD9HN0GdHrOrEUdJ/+uOfkg5EK5
uTvYJKsBdROgY2tS9wi4OARqfu0oEgorBgEEAZdVAQUBAQdABO0IRJ7sn6FMAYrb
DmDr6/wj+ijHsv6i+6xGDvIQsQIDAQgHiH4EGBYKACYWIQQ81JUQumxD/UTt+F7f
o+Ia96Y6jAUCan7tKAIbDAUJAeEzgAAKCRDfo+Ia96Y6jLTZAP4jQZJmp1n63o4L
tPCjR6SR3J1KuKb/fndRNO8usbEEfwD6AuzTzJCy3UKPyLzXI8dH5NETfOhFm8SW
ktUHmSvE4Ac=
=ra4y
-----END PGP PUBLIC KEY BLOCK-----```

### How to Encrypt Reports

```bash
# Encrypt your report with our public key
gpg --encrypt --recipient nablashell@gmail.com report.txt

# This creates report.txt.gpg which only we can decrypt
```

### Report Format
- Affected version(s)
- Description of vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (optional)

## Security Features

### Memory Protection
- Process memory isolation
- Permission-based access control
- Memory integrity checks
- Canary protection against overflows

### Anti-Analysis
- Anti-debug detection
- Obfuscated strings
- Encrypted pointers
- Dynamic key rotation

### Access Control
- Process ownership verification
- Read/write permission checks
- Heap/stack boundary validation

## Best Practices

### For Users
- Run with least privilege necessary
- Use isolated test environments
- Validate input configurations
- Monitor process access

### For Developers
- Follow secure coding guidelines
- Implement input validation
- Use memory-safe Rust patterns
- Handle errors gracefully

## Known Security Limitations

- Windows support incomplete
- UE4/5 signatures version-dependent
- Python process memory requires same user
- Some protections are Python-level only

## Security Updates

Updates are provided in release notes. Subscribe to releases for notifications.
