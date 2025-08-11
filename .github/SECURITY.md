# Security Policy

## Supported Versions

We actively support the following versions of Scriptoris with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of Scriptoris seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please send an email to: [security@scriptoris-project.org] (replace with actual email)

You should receive a response within 48 hours. If for some reason you do not, please follow up to ensure we received your original message.

### What to Include

Please include the following information in your report:

- Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit the issue

This information will help us triage your report more quickly.

### Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any potential similar problems
3. Prepare patches for all supported versions
4. Release patched versions as soon as possible
5. Publish a security advisory

### Safe Harbor

We support safe harbor for security researchers who:

- Make a good faith effort to avoid privacy violations, destruction of data, and interruption or degradation of our services
- Only interact with accounts you own or with explicit permission of the account holder
- Do not access a system beyond what is necessary to demonstrate a vulnerability
- Report any vulnerability you've discovered promptly
- Do not violate any other applicable laws or regulations

### Comments on This Policy

If you have suggestions on how this process could be improved please submit a pull request or file an issue to discuss.

## Security Best Practices for Users

### Installation

- Always download Scriptoris from official sources:
  - GitHub releases: https://github.com/yourusername/scriptoris/releases
  - Cargo: `cargo install scriptoris`
- Verify checksums/signatures when available

### Configuration

- Store configuration files in appropriate directories with correct permissions
- Avoid storing sensitive information in configuration files
- Review LSP server configurations and only use trusted language servers

### Usage

- Be cautious when editing files with elevated privileges
- Keep your Rust toolchain and dependencies up to date
- Report suspicious behavior or crashes that might indicate security issues

## Dependencies

Scriptoris relies on various third-party dependencies. We:

- Regularly audit our dependencies for known vulnerabilities
- Use automated tools to check for security issues
- Update dependencies promptly when security patches are available
- Pin dependency versions to ensure reproducible builds

If you discover a vulnerability in one of our dependencies, please:

1. Report it to the dependency's maintainers first
2. Inform us so we can track the issue and update when patches are available

## Security Considerations

### Terminal Security

As a terminal-based application, Scriptoris inherits the security model of the terminal environment:

- Terminal escape sequences are handled carefully to prevent injection attacks
- Input validation is performed on all user input
- File system access is restricted to user-accessible paths

### LSP Integration

The Language Server Protocol (LSP) integration in Scriptoris:

- Communicates with external language server processes
- Does not execute arbitrary code from language servers
- Validates all LSP messages and responses
- Runs language servers in separate processes

### Memory Safety

Being written in Rust, Scriptoris benefits from memory safety guarantees:

- No buffer overflows in safe Rust code
- Careful review of any unsafe code blocks
- Use of well-audited dependencies for low-level operations

---

**Note**: This security policy is subject to change. Please check back regularly for updates.