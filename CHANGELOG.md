# Changelog

All notable changes to Scriptoris will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2025-09-21

### Added
- 日本語ローカライズ（UI/ヘルプ/メッセージ）
- テーマ配色オプション（設定/UI 反映）

### Changed
- ロガー初期化を見直し（開発時デバッグ有効化）
- 依存バージョンを厳密固定（=x.y.z）
- ドキュメント更新とコードフォーマット調整

### Fixed
- エラーハンドリング強化（`unwrap` 削減）
- バッファ/ウィンドウ管理の安定化とUI反映
- CI/Clippy 警告を解消（`-D warnings`）
- セッション/マクロ/環境変数のテスト追加と安定化

### Fixed
- **Architecture Improvements**: Major refactoring to resolve borrow checker conflicts and improve code structure
  - Eliminated duplicate editor state between `App.editor` and `Buffer.content`
  - Unified editor access through `get_current_editor()` methods
  - Implemented Highlighter caching for better performance
  - Converted `CommandProcessor` to static methods to resolve borrowing issues
  - Improved UI rendering purity by removing state mutations during draw operations
- **Test Coverage**: Updated all tests to work with new architecture
- **Performance**: Reduced unnecessary Highlighter instantiations through caching
- **Code Quality**: Resolved complex borrowing conflicts and improved maintainability

## [0.1.0] - 2025-09-10

### Added
- **Core Editor Features**
  - Vim-style modal editing (Normal, Insert, Visual, Command modes)
  - Full Unicode and Japanese character support
  - Efficient text editing with Ropey rope data structure
  - Cross-platform terminal support (Windows, macOS, Linux)

- **Buffer Management**
  - Multiple buffer/tab support with `:b`, `:bn`, `:bp` commands
  - Buffer switching and management with `:ls`, `:bd`
  - Simultaneous editing of multiple files

- **Window Management**
  - Horizontal split windows (`:split`)
  - Vertical split windows (`:vsplit`)
  - Window navigation with `Ctrl+W` + direction keys
  - Recursive window splitting support

- **Session Management**
  - Save and restore workspace sessions (`:session save/load`)
  - JSON-based session persistence
  - Multiple named sessions support

- **LSP Integration**
  - Language Server Protocol plugin architecture
  - Built-in support for:
    - Rust (rust-analyzer)
    - TypeScript/JavaScript (typescript-language-server)  
    - Python (pylsp)
  - Features:
    - Code completion (`Ctrl+Space`)
    - Hover documentation (`Ctrl+K`)
    - Go to definition (`Ctrl+]`)
    - Real-time diagnostics
    - Document formatting

- **Markdown Support**
  - GitHub Flavored Markdown (GFM) processing
  - Support for tables, footnotes, strikethrough, task lists
  - HTML export with sanitization (mdcore crate)
  - Syntax-aware editing experience

- **Plugin Architecture**
  - Extensible plugin system with async support
  - Event-driven plugin hooks (key events, commands, file operations)
  - Clean plugin trait interface

- **Configuration**
  - JSON-based configuration system
  - Cross-platform config directory support
  - Customizable themes and editor settings
  - LSP server configuration

- **Advanced Text Editing**
  - Vim-style text objects and motions
  - Visual selection modes (character and line)
  - Copy/paste operations with internal clipboard
  - Undo/redo functionality
  - Replace mode support
  - Macro recording and playback

- **File Operations**
  - File open, save, and save-as operations
  - UTF-8 encoding support
  - File modification detection
  - Command-line file arguments

- **User Interface**
  - Terminal-based UI with Ratatui
  - Status bar with mode and file information
  - Command input with history
  - Help system with keybinding reference
  - Line numbers and cursor position display

### Technical Implementation
- **Architecture**: Rust workspace with multiple crates
  - `scriptoris`: Main TUI application
  - `lsp-plugin`: LSP integration and client
  - `mdcore`: Markdown processing library

- **Dependencies**:
  - Ratatui 0.26+ for terminal UI
  - Crossterm 0.27+ for cross-platform terminal handling
  - Ropey 1.6+ for efficient text editing
  - Comrak for Markdown processing
  - Tower-LSP for Language Server Protocol
  - Tokio for async runtime
  - Serde for configuration serialization

- **Performance**:
  - Efficient text handling for large files
  - Minimal memory footprint
  - Fast startup time
  - Responsive UI updates

### Platform Support
- **Linux**: Full support with comprehensive testing
- **macOS**: Full support including Apple Silicon
- **Windows**: Full support with proper terminal handling

---

## Release Notes Template

### [X.Y.Z] - YYYY-MM-DD

#### Added
- New features

#### Changed  
- Changes in existing functionality

#### Deprecated
- Soon-to-be removed features

#### Removed
- Now removed features

#### Fixed
- Bug fixes

#### Security
- Security improvements

---

## Development Milestones

### Planned for v0.2.0
- [ ] Enhanced LSP features (more language servers)
- [ ] Advanced Vim features (registers, macros improvements)
- [ ] Theme system with multiple built-in themes
- [ ] Plugin ecosystem and installation system
- [ ] Performance optimizations for very large files

### Planned for v0.3.0  
- [ ] Git integration (status, diff, commit)
- [ ] Search and replace with regex support
- [ ] File tree/explorer sidebar
- [ ] Split window improvements
- [ ] Configuration UI

### Future Considerations
- [ ] Remote editing capabilities
- [ ] Collaborative editing features
- [ ] Extended language support
- [ ] Custom syntax highlighting
- [ ] Plugin repository

---

**Legend:**
- 🆕 **Added** - New features
- 🔄 **Changed** - Changes in existing functionality  
- 🗑️ **Deprecated** - Soon-to-be removed features
- ❌ **Removed** - Now removed features
- 🐛 **Fixed** - Bug fixes
- 🔒 **Security** - Security improvements
