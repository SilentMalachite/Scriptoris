# Scriptoris - Terminal Markdown Editor

## Project Overview
Scriptoris is a Vim-style terminal Markdown editor built with Rust + Ratatui. It provides efficient Markdown editing in the terminal with cross-platform support (Windows/macOS/Linux).

## Architecture

### Workspace Structure
- `crates/scriptoris/` - Main TUI application
- `crates/mdcore/` - Markdown processing library
- `crates/lsp-plugin/` - LSP integration (future)

### Key Technologies
- **TUI Framework**: Ratatui 0.26 + Crossterm 0.27
- **Text Processing**: Ropey 1.6 for efficient text handling
- **Markdown**: comrak with GFM extensions (tables, footnotes, strikethrough, task lists)
- **Syntax Highlighting**: syntect 5.0
- **Unicode Support**: unicode-width, unicode-segmentation for Japanese text
- **Config**: serde/serde_json + directories for cross-platform settings
- **Async**: tokio 1.32 for file I/O

## Current Implementation Status

### Completed Features
- Basic TUI with title bar, editor area, status bar
- Vim-style modes (Normal/Insert/Command/Help)
- Text editing with Ropey backend
- Unicode/Japanese character support
- File operations (read/save)
- JSON configuration system
- Markdown processing with GFM support

### Key Components
- `app.rs` - Application state and mode management
- `editor.rs` - Text editing logic with Ropey integration
- `ui.rs` - UI rendering and layout
- `config.rs` - Settings management
- `mdcore/markdown.rs` - Markdown to HTML conversion

## Development Guidelines

### Code Style
- Follow Rust conventions and clippy recommendations
- Use anyhow for error handling
- Maintain cross-platform compatibility
- Preserve Unicode support throughout

### Testing
- Unit tests for core functionality
- Cross-platform compatibility testing
- Unicode/Japanese text handling verification

### Build Commands
```bash
# Development
cargo run
cargo run -- path/to/file.md

# Release
cargo build --release

# Testing
cargo test
```

## Future Roadmap
1. Enhanced Vim operations (visual mode, registers)
2. Syntax highlighting for Markdown
3. Search/replace functionality
4. LSP integration for advanced editing
5. External preview integration
6. Git integration

## Security Considerations
- HTML sanitization in mdcore
- Safe file operations with proper error handling
- No unsafe HTML rendering in Markdown processing
