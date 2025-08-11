# Scriptoris Demo Document

> 🚀 **Welcome to Scriptoris** - A powerful terminal-based Markdown editor with advanced features for developers and writers.

## Overview

**Scriptoris** combines the efficiency of Vim-style editing with modern development tools like Language Server Protocol (LSP) support, making it the ideal choice for editing documentation, code, and technical writing directly in your terminal.

## ✨ Core Features

### 🎯 Vim-Style Editing
Experience familiar modal editing with comprehensive keybinding support:

- **Normal Mode**: Navigate and manipulate text efficiently
- **Insert Mode**: Natural text input with all the typing comfort you expect  
- **Visual Mode**: Select text visually for operations
- **Command Mode**: Execute powerful editor commands

### 📁 Multiple Buffer Management
Work with multiple files simultaneously:

```
:e document1.md    # Open first document
:e document2.md    # Open second document  
:bn                # Switch to next buffer
:bp                # Switch to previous buffer
:ls                # List all open buffers
```

### 🪟 Split Window Support
Enhance your productivity with flexible window layouts:

| Command | Action | Description |
|---------|--------|-------------|
| `:split` | Horizontal Split | Split window horizontally |
| `:vsplit` | Vertical Split | Split window vertically |
| `Ctrl+W h/j/k/l` | Navigate | Move between split windows |

### 🔧 Language Server Protocol (LSP)
Get intelligent editing support for multiple programming languages:

#### Supported Languages
1. **Rust** 🦀 - Full rust-analyzer integration
2. **TypeScript/JavaScript** 📜 - Complete typescript-language-server support  
3. **Python** 🐍 - Comprehensive pylsp integration

#### LSP Features
- **Code Completion** (`Ctrl+Space`) - Smart autocomplete suggestions
- **Hover Documentation** (`Ctrl+K`) - Instant symbol information
- **Go to Definition** (`Ctrl+]`) - Jump to symbol definitions
- **Real-time Diagnostics** - Live error detection and warnings

### 💾 Session Management
Save and restore your entire workspace:

```bash
:session save my-project     # Save current session
:session load my-project     # Restore saved session
:session list               # View available sessions
```

## 📝 Advanced Markdown Support

### GitHub Flavored Markdown (GFM)
Full support for modern Markdown features:

- [x] **Tables** - Organize data clearly
- [x] **Task Lists** - Track progress visually  
- [x] **Footnotes** - Add detailed references¹
- [x] **Strikethrough** - ~~Mark completed items~~
- [x] **Code Blocks** - Syntax highlighted programming examples

### Code Examples with LSP

#### Rust Example
```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            features: HashMap::new(),
        }
    }
    
    // LSP provides intelligent completion here
    pub fn enable_feature(&mut self, feature: &str) {
        self.features.insert(feature.to_string(), true);
    }
}
```

#### TypeScript Example  
```typescript
interface EditorConfig {
    readonly theme: 'dark' | 'light';
    fontSize: number;
    enableLSP: boolean;
    languages: string[];
}

class Scriptoris {
    private config: EditorConfig;
    
    constructor(config: EditorConfig) {
        this.config = { ...config };
    }
    
    // Hover here shows full type information
    public updateConfig(updates: Partial<EditorConfig>): void {
        this.config = { ...this.config, ...updates };
    }
    
    // Go-to-definition works seamlessly
    public getLanguageSupport(lang: string): boolean {
        return this.config.languages.includes(lang);
    }
}
```

## 🎨 User Experience

### Cross-Platform Compatibility
- **Linux** 🐧 - Native terminal integration
- **macOS** 🍎 - Perfect Terminal.app and iTerm2 support
- **Windows** 🪟 - Full Windows Terminal compatibility

### Performance Characteristics
- **Fast Startup** ⚡ - Instant loading, even for large files
- **Memory Efficient** 💾 - Minimal resource usage
- **Responsive UI** 📱 - Smooth scrolling and navigation
- **Large File Support** 📊 - Handles files with 100k+ lines

### Keyboard Shortcuts Quick Reference

#### Movement
- `h/j/k/l` - Left/Down/Up/Right
- `w/b` - Word forward/backward  
- `0/$` - Line beginning/end
- `gg/G` - File beginning/end

#### Editing
- `i/a` - Insert before/after cursor
- `o/O` - Open line below/above
- `x/X` - Delete character forward/backward
- `dd` - Delete entire line
- `yy` - Yank (copy) line
- `p/P` - Paste after/before cursor

#### File Operations
- `:w` - Save file
- `:q` - Quit editor
- `:wq` - Save and quit
- `:e filename` - Open file

## 🚀 Getting Started

### Installation
```bash
# Build from source
git clone https://github.com/yourusername/scriptoris.git
cd scriptoris
cargo build --release

# Run directly
./target/release/scriptoris demo.md

# Or install globally
cargo install --path crates/scriptoris
```

### First Steps
1. **Open this demo file**: `scriptoris demo.md`
2. **Try basic navigation**: Use `h/j/k/l` to move around
3. **Enter insert mode**: Press `i` and start typing
4. **Return to normal mode**: Press `Esc`
5. **Save your changes**: Type `:w` and press Enter
6. **Get help**: Press `?` to see all keybindings

### Advanced Usage
1. **Open multiple files**: `:e another-file.md`
2. **Split windows**: `:vsplit` to see both files
3. **Try LSP features**: Open a `.rs` or `.ts` file and press `Ctrl+Space`
4. **Save your session**: `:session save demo-session`

## 🎯 Perfect For

### Technical Writers
- Documentation projects
- API documentation  
- Technical specifications
- Blog posts and articles

### Developers  
- README files
- Code documentation
- Project notes
- Configuration files

### Students & Researchers
- Academic papers
- Research notes  
- Study materials
- Thesis writing

## 🌟 What Makes Scriptoris Special

> **"The perfect blend of Vim efficiency and modern development tools"**

- **Terminal Native** - No GUI overhead, perfect for SSH and remote work
- **Extensible** - Plugin architecture for unlimited customization
- **Fast & Reliable** - Built with Rust for performance and safety
- **Developer Focused** - LSP integration brings IDE-like features to the terminal
- **Unicode Ready** - Full support for international text including Japanese: こんにちは

---

## Footnotes

¹ This is an example footnote demonstrating GFM support.

---

**Ready to experience the future of terminal-based editing?**

Try Scriptoris today and transform your writing workflow! 🚀✨

*This document showcases the capabilities of Scriptoris. Feel free to experiment with all the features described above.*