# Demo Content Creation Guide

This guide provides instructions for creating screenshots, demo videos, and promotional materials for Scriptoris.

## Assets Overview

This directory should contain:

```
assets/demo/
├── DEMO_GUIDE.md           # This file
├── screenshots/            # Static screenshots
│   ├── main-interface.png  # Main editor interface
│   ├── split-windows.png   # Window splitting demo
│   ├── lsp-completion.png  # LSP completion in action
│   ├── buffer-management.png # Multiple buffers
│   └── help-screen.png     # Help/keybinding reference
├── sample-files/           # Demo content files
│   ├── demo.md             # Main demo document
│   ├── code-sample.rs      # Rust code for LSP demo
│   ├── javascript-demo.js  # JS for LSP demo
│   └── complex-doc.md      # Complex markdown features
├── gifs/                   # Animated demonstrations
│   ├── basic-editing.gif   # Basic vim editing
│   ├── window-splits.gif   # Window management
│   ├── lsp-features.gif    # LSP in action
│   └── session-management.gif # Session save/load
└── videos/                 # Full demo videos (optional)
    └── full-demo.mp4       # Complete feature walkthrough
```

## Screenshot Guidelines

### General Requirements

- **Resolution**: Minimum 1920x1080 for high-DPI displays
- **Terminal Size**: 120x30 characters (readable text)
- **Color Scheme**: Use dark theme for consistency
- **Font**: Use a clear monospace font (e.g., Fira Code, JetBrains Mono)
- **Content**: Ensure demo content is professional and relevant

### Key Screenshots Needed

#### 1. Main Interface (`main-interface.png`)
- Basic editor with a markdown file open
- Show line numbers, status bar, mode indicator
- Cursor positioned in meaningful location
- Content should be the demo.md file

#### 2. Split Windows (`split-windows.png`)
- Horizontal and vertical splits
- Different files in each window
- Clear window borders/indicators
- Demonstrate the power of window management

#### 3. LSP Features (`lsp-completion.png`)
- Code completion popup visible
- Show hover documentation
- Rust or TypeScript file with meaningful code
- LSP diagnostics/errors visible

#### 4. Buffer Management (`buffer-management.png`)
- Multiple buffers/tabs visible
- Buffer list (`:ls` command output)
- Show switching between different file types

#### 5. Help Screen (`help-screen.png`)
- Full help/keybinding reference
- Show comprehensive command list
- Well-formatted, readable layout

### Terminal Setup for Screenshots

```bash
# Recommended terminal setup
export TERM=xterm-256color
resize -s 30 120  # 30 rows, 120 columns

# Set a nice color scheme (example for iTerm2/Terminal.app)
# Use a dark theme with good contrast

# Font recommendations:
# - Fira Code 14pt
# - JetBrains Mono 14pt
# - SF Mono 14pt
# - Cascadia Code 14pt
```

## Sample Files

### demo.md
Create a comprehensive markdown file showcasing:

```markdown
# Scriptoris Demo Document

This document demonstrates the capabilities of **Scriptoris**, a powerful terminal-based Markdown editor with advanced features.

## Core Features

### Text Editing
- **Vim-style editing** with familiar keybindings
- **Multiple buffers** for simultaneous file editing
- **Split windows** for enhanced productivity
- **Session management** to save and restore your workspace

### Markdown Support
- Full **GitHub Flavored Markdown** (GFM) support
- Tables, footnotes, strikethrough, and task lists
- Code blocks with syntax highlighting

| Feature | Status | Description |
|---------|---------|-------------|
| Vim Keybindings | ✅ | Complete modal editing support |
| LSP Integration | ✅ | Language server protocol support |
| Buffer Management | ✅ | Multiple file editing |
| Window Splitting | ✅ | Horizontal and vertical splits |

### Task List Example
- [x] Implement core editor functionality
- [x] Add LSP support for popular languages
- [x] Create comprehensive documentation
- [ ] Add plugin ecosystem
- [ ] Implement advanced themes

### Code Examples

```rust
// Rust code with LSP support
use ratatui::{Frame, layout::Rect};

pub struct Editor {
    cursor_line: usize,
    cursor_col: usize,
    content: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            cursor_line: 0,
            cursor_col: 0,
            content: String::new(),
        }
    }
}
```

```typescript
// TypeScript with intelligent completion
interface EditorConfig {
  theme: string;
  fontSize: number;
  tabSize: number;
  enableLSP: boolean;
}

class ScriptorisEditor {
  private config: EditorConfig;
  
  constructor(config: EditorConfig) {
    this.config = config;
  }
  
  public initialize(): Promise<void> {
    return new Promise((resolve) => {
      // Editor initialization logic
      resolve();
    });
  }
}
```

### Advanced Features

> **Note**: Scriptoris includes advanced features like LSP integration, providing intelligent code completion, hover documentation, and real-time diagnostics.

#### Supported Languages
1. **Rust** - via rust-analyzer
2. **TypeScript/JavaScript** - via typescript-language-server  
3. **Python** - via pylsp

#### Key Benefits
- 🚀 **Performance** - Built with Rust for speed
- 🎯 **Productivity** - Vim-style efficiency
- 🔧 **Extensibility** - Plugin architecture
- 🌍 **Cross-platform** - Works everywhere

---

*This document serves as a comprehensive demo of Scriptoris capabilities.*
```

### code-sample.rs
Create a Rust file for LSP demonstrations:

```rust
// Rust code sample for LSP demonstration
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub dependencies: HashMap<String, String>,
}

impl ProjectConfig {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            authors: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    pub fn add_dependency(&mut self, name: String, version: String) {
        self.dependencies.insert(name, version);
    }
    
    pub fn get_dependency(&self, name: &str) -> Option<&String> {
        self.dependencies.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_config_creation() {
        let config = ProjectConfig::new(
            "scriptoris".to_string(),
            "0.1.0".to_string()
        );
        
        assert_eq!(config.name, "scriptoris");
        assert_eq!(config.version, "0.1.0");
        assert!(config.authors.is_empty());
    }
}
```

### javascript-demo.js
Create a JavaScript file for LSP demonstrations:

```javascript
// JavaScript/TypeScript demo for LSP features
class DocumentEditor {
    constructor(config) {
        this.config = config;
        this.documents = new Map();
        this.activeDocument = null;
        this.plugins = [];
    }
    
    async openDocument(path) {
        try {
            const content = await this.loadFile(path);
            const document = new Document(path, content);
            this.documents.set(path, document);
            this.activeDocument = document;
            return document;
        } catch (error) {
            console.error(`Failed to open document: ${error.message}`);
            throw error;
        }
    }
    
    async loadFile(path) {
        // Simulated file loading
        return new Promise((resolve, reject) => {
            setTimeout(() => {
                resolve(`Content of ${path}`);
            }, 100);
        });
    }
    
    installPlugin(plugin) {
        if (typeof plugin.initialize === 'function') {
            plugin.initialize(this);
            this.plugins.push(plugin);
        } else {
            throw new Error('Plugin must implement initialize method');
        }
    }
    
    getActiveDocument() {
        return this.activeDocument;
    }
}

class Document {
    constructor(path, content) {
        this.path = path;
        this.content = content;
        this.modified = false;
        this.cursorPosition = { line: 0, column: 0 };
    }
    
    insertText(text, position = this.cursorPosition) {
        // Text insertion logic
        this.modified = true;
    }
    
    save() {
        // Save document logic
        this.modified = false;
    }
}

// Usage example
const editor = new DocumentEditor({
    theme: 'dark',
    fontSize: 14,
    enableLSP: true
});

editor.openDocument('README.md')
    .then(doc => {
        console.log('Document opened successfully');
    })
    .catch(err => {
        console.error('Error opening document:', err);
    });
```

## GIF Creation Guidelines

### Tools Recommended

- **macOS**: GIPHY Capture, Kap
- **Linux**: Peek, SimpleScreenRecorder + gifski
- **Windows**: ScreenToGif, LICEcap
- **Cross-platform**: OBS Studio + ffmpeg

### GIF Specifications

- **Duration**: 10-30 seconds per GIF
- **Frame Rate**: 15-20 fps (balance quality/size)
- **Resolution**: 1200x800 minimum
- **File Size**: Under 5MB per GIF
- **Loop**: Seamless loops where possible

### Key GIFs to Create

#### 1. Basic Editing (`basic-editing.gif`)
- Start with empty file
- Switch to insert mode (`i`)
- Type some text
- Navigate with `h/j/k/l`
- Delete text (`x`, `dd`)
- Show undo (`u`)

#### 2. Window Splits (`window-splits.gif`)  
- Start with one window
- Execute `:split` (horizontal)
- Execute `:vsplit` (vertical)
- Navigate between windows (`Ctrl+W` + direction)
- Show different files in each window

#### 3. LSP Features (`lsp-features.gif`)
- Open Rust or TypeScript file
- Show completion (`Ctrl+Space`)
- Demonstrate hover (`Ctrl+K`)
- Go to definition (`Ctrl+]`)
- Show diagnostics

#### 4. Session Management (`session-management.gif`)
- Open multiple files
- Create splits
- Save session (`:session save demo`)
- Close editor
- Restart and load session (`:session load demo`)

### Recording Tips

1. **Prepare**: Have all files ready, practice the sequence
2. **Clean Terminal**: Clear history, set consistent prompt
3. **Timing**: Allow pauses for viewers to read content
4. **Smooth Movements**: Don't rush cursor movements
5. **Reset State**: Start from clean state each recording

## Asset Management

### File Naming Convention

```
assets/demo/
├── screenshots/
│   ├── 01-main-interface.png
│   ├── 02-split-windows.png
│   ├── 03-lsp-completion.png
│   ├── 04-buffer-management.png
│   └── 05-help-screen.png
├── gifs/
│   ├── 01-basic-editing.gif
│   ├── 02-window-management.gif
│   ├── 03-lsp-features.gif
│   └── 04-session-management.gif
```

### Optimization

#### Images
```bash
# Optimize PNG files
pngcrush -reduce -brute original.png optimized.png

# Or using ImageOptim (macOS)
# Or TinyPNG online service
```

#### GIFs
```bash
# Optimize GIF files
gifsicle -O3 --colors 256 input.gif -o output.gif

# Convert video to optimized GIF
ffmpeg -i input.mp4 -vf "fps=15,scale=1200:-1:flags=lanczos,palettegen" palette.png
ffmpeg -i input.mp4 -i palette.png -filter_complex "fps=15,scale=1200:-1:flags=lanczos[x];[x][1:v]paletteuse" output.gif
```

## Usage in Documentation

### README.md
```markdown
![Scriptoris Demo](assets/demo/gifs/01-basic-editing.gif)

### Key Features

![Split Windows](assets/demo/screenshots/02-split-windows.png)
*Multiple windows and buffers for enhanced productivity*

![LSP Integration](assets/demo/screenshots/03-lsp-completion.png)
*Intelligent code completion and diagnostics*
```

### Social Media
- Use square crops (1:1) for Instagram
- Use landscape (16:9) for Twitter
- Include project branding/logo if available

---

## Checklist for Complete Demo Package

- [ ] All sample files created and polished
- [ ] Screenshots taken at proper resolution
- [ ] GIFs created showing key features
- [ ] All assets optimized for web use
- [ ] File names follow convention
- [ ] README.md updated with new assets
- [ ] Assets tested in GitHub markdown preview

This comprehensive demo package will showcase Scriptoris effectively and help attract users and contributors!