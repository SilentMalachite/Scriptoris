# Contributing to Scriptoris

Thank you for your interest in contributing to Scriptoris! This document provides guidelines and information for contributors.

## 🚀 Quick Start

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Create** a new branch for your feature/fix
4. **Make** your changes
5. **Test** your changes thoroughly
6. **Submit** a pull request

## 📋 Development Setup

### Prerequisites

- **Rust 1.70+** and Cargo
- **Git** for version control
- A **terminal emulator** that supports Unicode (for testing)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/your-username/scriptoris.git
cd scriptoris

# Build the project
cargo build

# Run tests
cargo test

# Run the editor
cargo run -- test.md
```

### Project Structure

```
scriptoris/
├── crates/
│   ├── scriptoris/          # Main TUI application
│   │   ├── src/
│   │   │   ├── main.rs      # Entry point
│   │   │   ├── app.rs       # Application state & logic
│   │   │   ├── editor.rs    # Text editing with Ropey
│   │   │   ├── ui.rs        # Ratatui UI rendering
│   │   │   └── ...          # Other modules
│   │   └── Cargo.toml
│   ├── lsp-plugin/          # LSP integration
│   │   ├── src/
│   │   │   ├── lib.rs       # Main plugin logic
│   │   │   ├── client.rs    # LSP client
│   │   │   ├── server.rs    # LSP server wrapper
│   │   │   └── ...          # LSP implementation
│   │   └── Cargo.toml
│   └── mdcore/              # Markdown processing
│       ├── src/
│       │   ├── lib.rs       # Public API
│       │   ├── markdown.rs  # Comrak integration
│       │   └── sanitize.rs  # HTML sanitization
│       └── Cargo.toml
├── .github/                 # GitHub Actions & templates
└── assets/                  # Static assets
```

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Specific crate tests
cargo test -p scriptoris
cargo test -p lsp-plugin
cargo test -p mdcore

# With debug output
RUST_LOG=debug cargo test

# Integration tests
cargo test --test integration
```

### Test Coverage

We aim for comprehensive test coverage. When adding new features:

1. **Unit tests** for individual functions/methods
2. **Integration tests** for module interactions  
3. **Manual testing** in various terminal environments

### Manual Testing Checklist

Before submitting a PR, please test:

- [ ] Basic editing (insert, delete, navigate)
- [ ] Vim keybindings work correctly
- [ ] File operations (open, save, new)
- [ ] Buffer management (multiple files)
- [ ] Window splitting and navigation
- [ ] LSP features (if applicable)
- [ ] Unicode/Japanese character support
- [ ] Cross-platform compatibility (if possible)

## 💡 Contributing Guidelines

### Code Style

- **Rust formatting**: Use `cargo fmt` before committing
- **Linting**: Ensure `cargo clippy` passes without warnings
- **Documentation**: Document public APIs with `///` comments
- **Error handling**: Use `anyhow` for error propagation
- **Async code**: Use `tokio` conventions

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code formatting (no functional changes)
- `refactor`: Code restructuring (no functional changes)
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(lsp): add hover documentation support
fix(editor): resolve cursor positioning bug in visual mode
docs(readme): update installation instructions
```

### Pull Request Process

1. **Branch naming**: Use descriptive names
   - `feature/add-macro-recording`
   - `fix/buffer-switching-bug`
   - `docs/improve-contributing-guide`

2. **PR Title**: Follow conventional commit format

3. **PR Description**: Include:
   - What changes were made and why
   - How to test the changes
   - Any breaking changes
   - Screenshots/demos if UI-related

4. **Review process**:
   - All PRs require review before merging
   - Address review feedback promptly
   - Keep PRs focused and reasonably sized

## 🎯 Areas for Contribution

### Good First Issues

- **Documentation improvements**
- **Test coverage expansion**
- **Minor UI/UX enhancements**
- **Bug fixes in existing features**

### Medium Complexity

- **New Vim keybindings/commands**
- **Theme and customization features**
- **Performance optimizations**
- **Additional LSP language support**

### Advanced Features

- **Plugin architecture enhancements**
- **Advanced text editing features**
- **Complex UI improvements**
- **New major features (after discussion)**

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment**: OS, terminal emulator, Rust version
2. **Steps to reproduce**: Clear, minimal reproduction steps  
3. **Expected behavior**: What should happen
4. **Actual behavior**: What actually happens
5. **Logs**: Any relevant debug output (`RUST_LOG=debug`)

Use the bug report template when creating issues.

## 💡 Feature Requests

For new features:

1. **Check existing issues** to avoid duplicates
2. **Provide use case**: Why is this feature needed?
3. **Describe behavior**: How should it work?
4. **Consider alternatives**: Are there other solutions?

Use the feature request template when creating issues.

## 📚 Resources

### Rust & Libraries

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Tutorial](https://ratatui.rs/tutorial/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
- [Ropey Documentation](https://docs.rs/ropey/)

### Editor Design

- [Vim Documentation](https://vimdoc.sourceforge.net/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [Terminal Capabilities](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)

### Architecture References

- Study existing terminal editors: [Helix](https://github.com/helix-editor/helix), [Xi](https://github.com/xi-editor/xi-editor)

## 🤝 Community

- **Be respectful** and constructive in all interactions
- **Help others** when you can
- **Ask questions** if anything is unclear
- **Follow** the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

## 🏷️ Release Process

Releases are managed by maintainers:

1. **Version bumping** follows [Semantic Versioning](https://semver.org/)
2. **Changelog** is updated for each release
3. **GitHub Releases** are created with binaries
4. **Crates.io** publication for stable releases

---

Thank you for contributing to Scriptoris! Your involvement helps make this a better tool for everyone. 🚀