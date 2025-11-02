# Scriptoris - Terminal Markdown Editor

[![Rust](https://img.shields.io/badge/rust-1.82.0-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/SilentMalachite/Scriptoris/workflows/CI/badge.svg)](https://github.com/SilentMalachite/Scriptoris/actions)
[![Version](https://img.shields.io/badge/version-0.1.4-blue.svg)](#)

> A powerful, Vim-inspired terminal Markdown editor built with Rust and Ratatui. 
> Designed for efficient text editing with cross-platform support, Unicode handling, and LSP integration.

## ✨ Features

### 🚀 Core Editor Features
- **Vim-style keybindings** - Complete modal editing (Normal/Insert/Visual/Command modes)
- **Powerful text engine** - Ropey-backed buffer with efficient text operations
- **Multiple buffers** - Edit multiple files simultaneously with `:b`, `:bn`, `:bp`
- **Split windows** - Horizontal (`:split`) and vertical (`:vsplit`) window management
- **Session management** - Save and restore workspace sessions with `:session save/load`
- **Unicode support** - Full Japanese and international character support with proper grapheme handling

### 🔧 Advanced Features
- **LSP integration** - Language Server Protocol support with completion, hover, and diagnostics
  - Rust (rust-analyzer)
  - TypeScript/JavaScript (typescript-language-server)
  - Python (pylsp)
- **Plugin architecture** - Extensible async plugin system with event hooks
- **Macro recording** - Record and replay command sequences (`q<register>`, `@<register>`)
- **Syntax highlighting** - Context-aware highlighting via syntect
- **Configuration** - JSON-based settings with environment variable overrides

### 📝 Markdown Processing
- **GitHub Flavored Markdown** (GFM) support via Comrak
- **HTML export** with sanitization (mdcore crate)
- **Extended syntax** - Tables, footnotes, strikethrough, task lists
- **Math blocks** - LaTeX math detection (experimental)
- **Mermaid diagrams** - Diagram block detection (experimental)

### 🎨 User Experience
- **Cross-platform** - Native support for Windows, macOS, and Linux
- **Lightweight** - Minimal resource usage, ideal for SSH/remote editing
- **Fast startup** - Instant loading with 6500+ lines of optimized Rust code
- **Customizable** - Themes, keybindings, and editor settings via JSON
- **Japanese localization** - UI and help text available in Japanese

## 🚀 Quick Start

### Installation

**Prerequisites:** Rust 1.82+ and Cargo

```bash
# Clone and build
git clone https://github.com/SilentMalachite/Scriptoris.git
cd Scriptoris
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

### Command Mode (`:` prefix)
| Command | Description |
|---------|-------------|
| `:w` | Save current buffer |
| `:w <path>` | Save buffer as `<path>` |
| `:wq` | Save and quit |
| `:q` | Quit (fails if modified) |
| `:q!` | Force quit discarding changes |
| `:e <path>` | Open file in current buffer |
| `:b <n>` | Switch to buffer `<n>` |
| `:bn` / `:bp` | Next/previous buffer |
| `:ls` | List all buffers |
| `:bd` | Close current buffer |
| `:split` / `:vsplit` | Horizontal/vertical split |
| `:session save <name>` | Save current session |
| `:session load <name>` | Load saved session |
| `:set theme <name>` | Change syntax theme |
| `:search <term>` | Search for text |

### LSP Features
| Keybinding | Action |
|------------|--------|
| `Ctrl+Space` | Trigger completion |
| `Ctrl+K` | Show hover documentation |
| `Ctrl+]` | Go to definition |
| Auto | Real-time diagnostics |

### Macro Recording
| Key | Action |
|-----|--------|
| `q<register>` | Start/stop recording to `<register>` |
| `@<register>` | Replay macro from `<register>` |

## 📁 Project Structure

```
Scriptoris/
├── crates/
│   ├── scriptoris/            # Main TUI application (~6500 LOC)
│   │   ├── src/
│   │   │   ├── main.rs        # Entry point, terminal initialization
│   │   │   ├── app.rs         # Application state, mode management
│   │   │   ├── editor.rs      # Text editing with Ropey
│   │   │   ├── ui.rs          # Ratatui UI rendering
│   │   │   ├── enhanced_ui.rs # Advanced UI components
│   │   │   ├── command_processor.rs  # Command execution
│   │   │   ├── file_manager.rs       # File I/O operations
│   │   │   ├── session_manager.rs    # Session persistence
│   │   │   ├── config.rs      # Configuration management
│   │   │   ├── highlight.rs   # Syntax highlighting
│   │   │   ├── text_width.rs  # Unicode width calculations
│   │   │   ├── ui_state.rs    # UI state management
│   │   │   └── status_manager.rs     # Status messages
│   │   └── Cargo.toml
│   ├── lsp-plugin/            # LSP client implementation
│   │   ├── src/
│   │   │   ├── lib.rs         # Plugin interface
│   │   │   ├── client.rs      # LSP client with Tower-LSP
│   │   │   └── document.rs    # UTF-16 offset handling
│   │   └── Cargo.toml
│   └── mdcore/                # Markdown processing library
│       ├── src/
│       │   ├── lib.rs         # Public API
│       │   ├── markdown.rs    # Comrak GFM processing
│       │   ├── sanitize.rs    # Ammonia HTML sanitization
│       │   └── tests.rs       # Test utilities
│       └── Cargo.toml
├── assets/                    # Static resources
├── .github/                   # CI/CD workflows
├── Cargo.toml                 # Workspace configuration
├── rust-toolchain.toml        # Rust 1.82.0
├── README.md                  # This file
├── CONTRIBUTING.md            # Contribution guidelines
├── DEVELOPMENT.md             # Developer documentation
├── CHANGELOG.md               # Version history
└── LICENSE                    # MIT License
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

### Environment Tips

開発時にユーザー設定やセッションディレクトリを分離したい場合は、以下の環境変数を利用できます。

- `SCRIPTORIS_CONFIG_PATH` — 設定ファイルの絶対パスを直接指定（最優先）
- `SCRIPTORIS_CONFIG_DIR` — `config.json` を含むディレクトリを指定
- `SCRIPTORIS_DATA_DIR` — セッション JSON を保存するディレクトリを指定

CI やローカルテストでは一時ディレクトリを割り当てることで、既存ユーザー設定を書き換えずに動作検証できます。

### Architecture Overview

- **scriptoris** - Main TUI application (~6500 LOC)
  - Ratatui 0.26+ for terminal UI
  - Crossterm 0.27+ for cross-platform terminal handling
  - Ropey 1.6+ for efficient text buffer management
  - Syntect 5.0+ for syntax highlighting
  - Tokio async runtime for I/O operations
- **lsp-plugin** - LSP client with Tower-LSP
  - JSON-RPC communication with language servers
  - UTF-16 offset handling for Unicode text
  - Async document synchronization
- **mdcore** - Markdown processing library
  - Comrak 0.29+ for GFM parsing
  - Ammonia 4.1+ for HTML sanitization
  - Support for tables, footnotes, task lists

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

Environment overrides (開発・テスト向け):
- `SCRIPTORIS_CONFIG_PATH` — 設定ファイルへの完全なパスを指定（`SCRIPTORIS_CONFIG_DIR` より優先）
- `SCRIPTORIS_CONFIG_DIR` — `config.json` を含むディレクトリを指定（`config.json` が自動連結）
- `SCRIPTORIS_DATA_DIR` — セッションなどのデータを保存するルートディレクトリを指定

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
- **Unicode Support**: Terminal emulator compatibility varies; Scriptoris uses grapheme cluster-aware width calculations for proper cursor positioning
- **LSP Dependencies**: Requires external language servers (rust-analyzer, typescript-language-server, pylsp) to be installed separately
- **Terminal-Only**: No GUI preview mode by design (terminal-focused workflow)
- **Vim Compatibility**: Core Vim features implemented; some advanced features (complex registers, ex commands) may differ
- **Math/Mermaid**: Detection only; rendering requires external tools

## 🗺️ Roadmap

### v0.2.0 (Planned)
- [ ] **Enhanced LSP Features** - More language servers, improved diagnostics panel
- [ ] **Advanced Vim Features** - More text objects, improved visual mode
- [ ] **Theme System** - Multiple built-in themes with hot-reload
- [ ] **Search & Replace** - Regex support, visual feedback
- [ ] **Performance** - Optimizations for files >100k lines

### v0.3.0 (Future)
- [ ] **Git Integration** - Status, diff, commit from editor
- [ ] **File Explorer** - Sidebar tree view with navigation
- [ ] **Plugin Ecosystem** - Plugin repository and installation
- [ ] **Enhanced UI** - Preview pane, floating windows
- [ ] **Collaboration** - Remote editing capabilities

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built with excellent Rust libraries:
- [Ratatui](https://github.com/ratatui-org/ratatui) 0.26+ - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) 0.27+ - Cross-platform terminal manipulation
- [Ropey](https://github.com/cessen/ropey) 1.6+ - Efficient rope-based text buffer
- [Comrak](https://github.com/kivikakk/comrak) 0.29+ - GitHub Flavored Markdown parser
- [Ammonia](https://github.com/rust-ammonia/ammonia) 4.1+ - HTML sanitization
- [Tower-LSP](https://github.com/ebkalderon/tower-lsp) - Language Server Protocol implementation
- [Syntect](https://github.com/trishume/syntect) 5.0+ - Syntax highlighting engine
- [Tokio](https://github.com/tokio-rs/tokio) 1.47+ - Async runtime

---

**Scriptoris** - *Write better documentation, faster.* 📝✨

For questions, bug reports, or feature requests, please [open an issue](https://github.com/SilentMalachite/Scriptoris/issues).
