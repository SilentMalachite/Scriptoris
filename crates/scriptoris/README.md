# Scriptoris - Terminal-based Markdown Editor

A lightweight, terminal-based Markdown editor with nano-style keybindings and JSON configuration.

## Features

- **Nano-style keybindings** - Familiar keyboard shortcuts for users of GNU nano
- **Syntax highlighting** - Powered by syntect
- **Markdown support** - Full GFM support via mdcore
- **Configurable** - JSON-based configuration for themes, fonts, and keybindings
- **Fast and lightweight** - Built with Rust for optimal performance
- **Cross-platform** - Works on Windows, macOS, and Linux

## Installation

```bash
# Build from source
cargo build --release --bin scriptoris

# Run directly
cargo run --bin scriptoris

# Or install globally
cargo install --path crates/scriptoris
```

## Usage

### Basic Commands

Start the editor:
```bash
scriptoris [filename]
```

### Nano-style Keybindings

| Key       | Action                |
|-----------|-----------------------|
| `Ctrl+G`  | Get Help             |
| `Ctrl+O`  | Write Out (Save)     |
| `Ctrl+R`  | Read File (Open)     |
| `Ctrl+X`  | Exit                 |
| `Ctrl+W`  | Where Is (Search)    |
| `Ctrl+K`  | Cut Text (Cut line)  |
| `Ctrl+U`  | Uncut Text (Paste)   |
| `Ctrl+C`  | Current Position     |
| `Ctrl+V`  | Next Page            |
| `Ctrl+Y`  | Previous Page        |

### Navigation

- **Arrow keys** - Move cursor
- **Home/End** - Beginning/end of line
- **PageUp/PageDown** - Scroll pages

## Configuration

Configuration is stored in JSON format at:
- **Linux/macOS**: `~/.config/scriptoris-tui/config.json`
- **Windows**: `%APPDATA%\scriptoris-tui\config.json`

### Example Configuration

```json
{
  "theme": {
    "name": "dark",
    "syntax_theme": "base16-ocean.dark"
  },
  "font": {
    "size": 14,
    "family": "monospace"
  },
  "editor": {
    "tab_size": 4,
    "use_spaces": true,
    "line_numbers": true,
    "highlight_current_line": true,
    "wrap_lines": false
  },
  "keybindings": "Nano"
}
```

### Available Themes

The editor uses syntect themes. Popular options include:
- `base16-ocean.dark`
- `base16-ocean.light`
- `Solarized (dark)`
- `Solarized (light)`
- `Monokai`
- `Tomorrow`

### Keybinding Styles

Three keybinding styles are available:
- `"Nano"` - Default, GNU nano compatible
- `"Vim"` - Basic vim keybindings
- `"Emacs"` - Basic emacs keybindings

## Custom Keybindings

You can define custom keybindings in the configuration:

```json
{
  "keybindings": [
    {
      "key": "s",
      "modifiers": ["ctrl"],
      "action": "save"
    },
    {
      "key": "f",
      "modifiers": ["ctrl"],
      "action": "search"
    }
  ]
}
```

## Building from Source

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Build Commands

```bash
# Debug build
cargo build --bin tui_app

# Release build (optimized)
cargo build --release --bin scriptoris

# Run tests
cargo test --bin scriptoris

# Run with logging
RUST_LOG=debug cargo run --bin scriptoris
```

## Troubleshooting

### Terminal Issues

If the display appears corrupted:
1. Ensure your terminal supports UTF-8
2. Try a different terminal emulator
3. Check your `TERM` environment variable

### Performance

For large files, consider:
- Disabling line numbers in config
- Turning off syntax highlighting
- Using release build (`--release`)

## License

MIT License - See LICENSE file for details