# Scriptoris - Terminal Markdown Editor

[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/yourusername/scriptoris/workflows/CI/badge.svg)](https://github.com/yourusername/scriptoris/actions)
[![Tests](https://img.shields.io/badge/tests-56_passing-brightgreen.svg)](#)

> A focused, Vim-inspired terminal Markdown editor built with Rust. 
> Optimised for fast note taking and documentation with a lean, easy-to-understand codebase.

![Scriptoris Demo](assets/demo.gif)
*Coming soon: Demo gif showing key features*

## ✨ Features

### 🚀 Core Editor Features
- **Vim-style keybindings** - Familiar modal editing (Normal/Insert/Visual/Command modes)
- **Powerful text engine** - Ropey-backed buffer for smooth editing of large Markdown files
- **Search command** - `:search <term>` quickly moves the cursor to matches in the current file
- **Markdown aware UI** - Syntax-highlighted viewport using syntect + ratatui
- **Unicode support** - Full Japanese and international character support

### 🔧 Advanced Features
- **Prototype LSP integration** *(optional)* - Example plugin demonstrating completion & hover hooks
- **Extensible architecture** - Traits for future plugin loading (currently disabled by default)
- **Markdown processing library** - `mdcore` crate for sanitised HTML export and utilities

### 📝 Markdown & Highlighting
- **GitHub Flavored Markdown** (GFM) support in the renderer tests
- **Syntax highlighting** via syntect with Markdown-first heuristics
- **HTML export** and sanitisation powered by the companion `mdcore` crate

### 🎨 User Experience
- **Cross-platform** - Works on Windows, macOS, and Linux
- **Lightweight** - Minimal resource usage, perfect for SSH/remote editing
- **Fast startup** - Instant loading even for large files
- **Customizable** - JSON configuration for themes and keybindings

## 🚀 Quick Start

### Installation

**Prerequisites:** Rust 1.82+ and Cargo

```bash
# Clone and build
git clone https://github.com/yourusername/scriptoris.git
cd scriptoris
cargo build --release

# Run directly
cargo run -- document.md

# Install globally
cargo install --path crates/scriptoris
```

### Basic Usage

```bash
# Start with a new file
scriptoris

# Open an existing file
scriptoris README.md

# (Planned) Multi-file support
# Current builds focus on a single active buffer.
```

## 🎯 Quick Reference

### Vim-style Modes & Navigation
| Key | Action | Mode |
|-----|--------|------|
| `h/j/k/l` | Move cursor left/down/up/right | Normal |
| `i` | Enter insert mode | Normal |
| `v` | Enter visual mode | Normal |
| `:` | Enter command mode | Normal |
| `Esc` | Return to normal mode | Any |

### Command Palette (Normal/Command Modes)
| Command | Description |
|---------|-------------|
| `:w` | Save to current file |
| `:w <path>` | Save buffer as `<path>` |
| `:wq` | Save and quit |
| `:q` | Quit (fails if buffer is modified) |
| `:q!` | Force quit discarding changes |
| `:e <path>` | Load a file into the current buffer |
| `:search <term>` | Jump cursor to the next occurrence of `<term>` |
| `:set theme <name>` | Change the active syntax highlighting theme |

### Experimental LSP Plugin
- The `crates/lsp-plugin` crate bundles a demo client/server showcasing how completions and hover
  information could be wired. It is **not** connected to real language servers yet, but serves as a
  reference for contributors interested in extending Scriptoris.

## 📁 Project Structure

```
scriptoris/
├── crates/
│   ├── scriptoris/     # Main TUI application (this crate)
│   ├── lsp-plugin/     # Prototype LSP client/server for future integration
│   └── mdcore/         # Markdown processing & sanitisation helpers
├── assets/             # Static assets and themes
├── .github/            # GitHub Actions and templates
└── README.md           # This file
```

## 🛠️ Development

### Building from Source

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Architecture Overview

- **scriptoris** - Main TUI application using Ratatui + Crossterm
- **lsp-plugin** - LSP client implementation with Tower-LSP
- **mdcore** - Markdown processing with Comrak + Ammonia sanitization

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📖 Configuration

Configuration file location:
- **Linux/macOS**: `~/.config/scriptoris/config.json`
- **Windows**: `%APPDATA%\scriptoris\config.json`

### Example Configuration

```json
{
  "theme": {
    "name": "dark",
    "syntax_theme": "base16-ocean.dark"
  },
  "editor": {
    "tab_size": 4,
    "use_spaces": true,
    "line_numbers": true,
    "wrap_lines": false
  },
  "lsp": {
    "auto_start": true,
    "show_diagnostics_inline": true,
    "show_hover_documentation": true
  },
  "keybindings": "vim"
}
```

## 🐛 Known Issues & Limitations

- **Performance**: Large files (>100k lines) may experience performance degradation
- **Unicode Support**: Some terminal emulators may not support all Unicode characters perfectly; Scriptoris adjusts cursor position using Unicode display width for better alignment
- **LSP Dependencies**: LSP features require external language servers to be installed
- **Terminal-Only**: No GUI preview mode (terminal-only by design)
- **Buffer/Window Commands**: Some advanced buffer and window management commands are temporarily disabled due to recent architecture improvements (will be restored in next release)

## 🗺️ Roadmap

- [ ] **Enhanced LSP Features** - More language servers, better diagnostics UI
- [ ] **Advanced Vim Features** - Macros, registers, more text objects
- [ ] **Plugin Ecosystem** - Plugin repository and installation system
- [ ] **Themes & Customization** - More built-in themes, better customization
- [ ] **Performance Optimization** - Better handling of very large files
- [ ] **Git Integration** - Built-in git status, diff, and commit features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [Ropey](https://github.com/cessen/ropey) - Efficient text buffer
- [Comrak](https://github.com/kivikakk/comrak) - CommonMark parser
- [Tower-LSP](https://github.com/ebkalderon/tower-lsp) - Language Server Protocol implementation
- [Syntect](https://github.com/trishume/syntect) - Syntax highlighting

---

**Scriptoris** - *Write better documentation, faster.* 📝✨

For questions, bug reports, or feature requests, please [open an issue](https://github.com/yourusername/scriptoris/issues).
