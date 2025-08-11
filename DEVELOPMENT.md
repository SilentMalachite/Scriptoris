# Development Guide

This guide provides detailed instructions for setting up a development environment for Scriptoris and contributing to the project.

## Quick Start

```bash
git clone https://github.com/yourusername/scriptoris.git
cd scriptoris
cargo build
cargo run -- test.md
```

## Development Environment Setup

### Prerequisites

#### Required

- **Rust 1.70.0+** - Install via [rustup](https://rustup.rs/)
- **Git** - Version control
- **A terminal emulator** that supports Unicode

#### Recommended

- **IDE/Editor** with Rust support:
  - [VS Code](https://code.visualstudio.com/) with [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
  - [RustRover](https://www.jetbrains.com/rust/) (JetBrains)
  - [Neovim](https://neovim.io/) with LSP support
  - [Helix](https://helix-editor.com/) (built-in LSP)

### Rust Installation

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH (or restart terminal)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install useful components
rustup component add rustfmt clippy
```

### Development Tools

Install additional tools for development:

```bash
# Code coverage
cargo install cargo-llvm-cov

# Security audit
cargo install cargo-audit

# Dependency tree visualization
cargo install cargo-tree

# Watch for changes and auto-rebuild
cargo install cargo-watch

# License checking
cargo install cargo-license

# Unused dependency detection
cargo install cargo-machete
```

### LSP Servers (for testing LSP features)

Install language servers to test the LSP plugin:

```bash
# Rust
rustup component add rust-analyzer

# TypeScript/JavaScript (requires Node.js)
npm install -g typescript-language-server typescript

# Python
pip install python-lsp-server[all]
```

## Project Structure Deep Dive

```
scriptoris/
├── .github/                    # GitHub workflows and templates
│   ├── workflows/
│   │   ├── ci.yml             # Continuous integration
│   │   └── release.yml        # Release automation
│   ├── ISSUE_TEMPLATE/        # Issue templates
│   └── pull_request_template.md
├── assets/                    # Static assets (future use)
├── crates/                    # Rust workspace crates
│   ├── scriptoris/           # Main TUI application
│   │   ├── src/
│   │   │   ├── main.rs       # Application entry point
│   │   │   ├── app.rs        # Core application state
│   │   │   ├── editor.rs     # Text editing with Ropey
│   │   │   ├── ui.rs         # Terminal UI with Ratatui
│   │   │   ├── command_processor.rs  # Command handling
│   │   │   ├── file_manager.rs      # File I/O operations
│   │   │   ├── config.rs     # Configuration management
│   │   │   ├── ui_state.rs   # UI state management
│   │   │   ├── status_manager.rs    # Status messages
│   │   │   └── lib.rs        # Library exports
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lsp-plugin/           # LSP integration
│   │   ├── src/
│   │   │   ├── lib.rs        # Plugin interface
│   │   │   ├── client.rs     # LSP client implementation
│   │   │   ├── server.rs     # LSP server wrapper
│   │   │   ├── document.rs   # Document synchronization
│   │   │   ├── capabilities.rs      # LSP capabilities
│   │   │   └── plugin.rs     # Scriptoris plugin trait impl
│   │   └── Cargo.toml
│   └── mdcore/               # Markdown processing
│       ├── src/
│       │   ├── lib.rs        # Public API
│       │   ├── markdown.rs   # Comrak integration
│       │   ├── sanitize.rs   # HTML sanitization
│       │   └── tests.rs      # Test utilities
│       └── Cargo.toml
├── scripts/                  # Build and development scripts
├── docs/                     # Additional documentation (future)
├── Cargo.toml               # Workspace configuration
├── rust-toolchain.toml      # Rust version specification
├── README.md                # Project overview
├── CONTRIBUTING.md          # Contribution guide
├── CHANGELOG.md             # Version history
├── LICENSE                  # MIT license
├── DEVELOPMENT.md           # This file
└── RELEASE_PROCESS.md       # Release procedures
```

### Key Modules

#### `scriptoris/src/app.rs`
- **Purpose**: Core application state and logic
- **Key types**: `App`, `Mode`, `BufferManager`, `WindowManager`
- **Responsibilities**: Vim mode handling, buffer/window management, plugin coordination

#### `scriptoris/src/editor.rs`
- **Purpose**: Text editing functionality
- **Key type**: `Editor`
- **Technology**: Ropey rope data structure
- **Features**: Cursor management, text operations, undo/redo

#### `scriptoris/src/ui.rs`
- **Purpose**: Terminal user interface
- **Technology**: Ratatui + Crossterm
- **Responsibilities**: Rendering, layout, event handling

#### `lsp-plugin/src/`
- **Purpose**: Language Server Protocol integration
- **Technology**: Tower-LSP, JSON-RPC
- **Features**: Completion, hover, diagnostics, go-to-definition

#### `mdcore/src/`
- **Purpose**: Markdown processing
- **Technology**: Comrak + Ammonia
- **Features**: GFM parsing, HTML generation, sanitization

## Development Workflow

### 1. Setting up for Development

```bash
# Clone the repository
git clone https://github.com/yourusername/scriptoris.git
cd scriptoris

# Create a feature branch
git checkout -b feature/my-awesome-feature

# Build and test
cargo build
cargo test
```

### 2. Running During Development

```bash
# Run with a test file
cargo run -- test.md

# Run with debug logging
RUST_LOG=debug cargo run -- test.md

# Run specific crate
cargo run -p scriptoris -- test.md

# Auto-rebuild on changes (requires cargo-watch)
cargo watch -x 'run -- test.md'
```

### 3. Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p scriptoris
cargo test -p lsp-plugin
cargo test -p mdcore

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_buffer_management

# Integration tests
cargo test --test integration

# Run tests with coverage
cargo llvm-cov --html
```

### 4. Code Quality

```bash
# Format code
cargo fmt

# Check formatting (CI style)
cargo fmt --all -- --check

# Run linter
cargo clippy

# Lint with all features
cargo clippy --all-targets --all-features

# Security audit
cargo audit

# Check for unused dependencies
cargo machete
```

### 5. Documentation

```bash
# Generate and open documentation
cargo doc --open

# Build documentation without dependencies
cargo doc --no-deps

# Check documentation
cargo doc --all --no-deps
```

## Debugging

### Logging

Scriptoris uses the `log` crate for logging:

```rust
use log::{debug, info, warn, error};

fn some_function() {
    debug!("Debug information");
    info!("General information");
    warn!("Warning message");
    error!("Error occurred");
}
```

Run with logging:
```bash
# All logs
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=scriptoris::editor=debug cargo run

# Multiple levels
RUST_LOG=scriptoris=debug,lsp_plugin=info cargo run
```

### Debugging Techniques

#### 1. Terminal Debugging

Since Scriptoris is a terminal application, debugging requires special techniques:

```rust
// Use eprintln! for debugging (goes to stderr)
eprintln!("Debug: cursor position = {:?}", cursor_pos);

// Or use a log file
use std::fs::OpenOptions;
use std::io::Write;

let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("/tmp/scriptoris_debug.log")?;
writeln!(file, "Debug: {}", message)?;
```

#### 2. Testing UI Components

Create unit tests for UI components:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_ui_rendering() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            draw_editor(f, &app);
        }).unwrap();
        
        let buffer = terminal.backend().buffer();
        // Assert buffer contents
    }
}
```

#### 3. LSP Debugging

For LSP issues, enable JSON-RPC tracing:

```bash
RUST_LOG=tower_lsp=debug,lsp_plugin=debug cargo run
```

### Common Issues and Solutions

#### Issue: Terminal Display Problems
- **Solution**: Test with different terminal emulators
- **Debug**: Check Unicode support, color capabilities

#### Issue: LSP Not Working
- **Solution**: Verify language server installation
- **Debug**: Check LSP server logs, JSON-RPC communication

#### Issue: Performance Issues
- **Solution**: Profile with `perf` or `cargo flamegraph`
- **Debug**: Use `cargo bench` for benchmarking

## Testing Guidelines

### Unit Tests

Test individual functions and components:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        let mut editor = Editor::new();
        editor.move_cursor_right();
        assert_eq!(editor.cursor_col(), 1);
    }
}
```

### Integration Tests

Test component interactions:

```rust
// tests/integration.rs
use scriptoris::{App, Mode};

#[tokio::test]
async fn test_file_operations() {
    let mut app = App::new().await.unwrap();
    // Test file loading, editing, saving
}
```

### Manual Testing Checklist

Before submitting PRs, test:

- [ ] **Basic Editing**: Insert, delete, navigate
- [ ] **Vim Keybindings**: All modal operations
- [ ] **File Operations**: Open, save, new file
- [ ] **Buffer Management**: Multiple files, switching
- [ ] **Window Operations**: Split, navigate
- [ ] **LSP Features**: Completion, hover, diagnostics
- [ ] **Unicode Support**: Japanese characters, emojis
- [ ] **Error Handling**: Invalid files, permission errors
- [ ] **Performance**: Large files (10k+ lines)

### Platform Testing

Test on multiple platforms when possible:

- **Linux**: Ubuntu 22.04+, Arch Linux
- **macOS**: macOS 12+ (Intel and Apple Silicon)
- **Windows**: Windows 10+

## Performance Considerations

### Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Profile CPU usage
cargo flamegraph --bin scriptoris -- large_file.md

# Memory profiling with valgrind (Linux)
cargo build
valgrind --tool=massif target/debug/scriptoris test.md
```

### Optimization Guidelines

1. **Text Handling**: Use Ropey efficiently
   - Avoid unnecessary string allocations
   - Use slices where possible
   - Cache cursor positions

2. **UI Rendering**: Minimize redraws
   - Only update changed areas
   - Use efficient layout algorithms
   - Batch terminal operations

3. **LSP Communication**: Optimize message passing
   - Debounce rapid changes
   - Cache completion results
   - Use incremental synchronization

## Architecture Guidelines

### Code Organization

- **Separation of Concerns**: UI, logic, data separate
- **Async/Await**: Use for I/O operations
- **Error Handling**: Use `anyhow` for error propagation
- **Configuration**: JSON-based, validated with `serde`

### Design Patterns

- **Plugin Architecture**: Trait-based extensibility
- **State Management**: Centralized application state
- **Event Handling**: Async event processing
- **Resource Management**: RAII patterns

### Dependencies

- **Minimize Dependencies**: Only add when necessary
- **Security**: Regular audits with `cargo audit`
- **Compatibility**: Maintain MSRV (Minimum Supported Rust Version)
- **Licensing**: Ensure compatible licenses

## Contributing Workflow

1. **Fork** the repository
2. **Create** feature branch
3. **Develop** with tests
4. **Document** changes
5. **Test** thoroughly
6. **Submit** pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Troubleshooting

### Build Issues

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update

# Check for dependency conflicts
cargo tree --duplicates
```

### Runtime Issues

```bash
# Debug run with logging
RUST_LOG=debug cargo run 2>&1 | tee debug.log

# Check system dependencies
ldd target/debug/scriptoris  # Linux
otool -L target/debug/scriptoris  # macOS
```

### IDE Issues

- **rust-analyzer**: Restart the language server
- **VS Code**: Reload window, check extension versions
- **Path Issues**: Ensure Rust/Cargo in PATH

---

Happy coding! 🦀