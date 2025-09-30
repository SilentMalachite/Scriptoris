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
- `editor.rs` - Text editing logic with Ropey integration and Unicode support
- `ui.rs` / `enhanced_ui.rs` - UI rendering and layout (basic/enhanced modes)
- `config.rs` - Settings management
- `text_width.rs` - Cross-platform Unicode width calculation
- `file_manager.rs` - File operations with robust error handling
- `lsp-plugin/` - LSP integration (async with proper cleanup)
- `mdcore/markdown.rs` - Markdown to HTML conversion with GFM support

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

## Recent Improvements
1. ✅ UTF-16 offset calculation with grapheme cluster support (LSP)
2. ✅ Enhanced Unicode processing (combining characters, emoji)
3. ✅ Memory leak prevention (LSP client cleanup, timeout handling)
4. ✅ Robust error handling (retry with exponential backoff)
5. ✅ Comprehensive test coverage (LSP, enhanced UI, Japanese text)

## Future Roadmap
1. Enhanced Vim operations (more registers, macros)
2. Advanced search/replace (regex support)
3. LSP features (code actions, refactoring)
4. External preview integration
5. Git integration
6. Plugin system

## Security Considerations
- HTML sanitization in mdcore
- Safe file operations with proper error handling
- No unsafe HTML rendering in Markdown processing
