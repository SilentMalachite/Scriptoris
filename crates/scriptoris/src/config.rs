use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub font: FontConfig,
    pub editor: EditorConfig,
    pub keybindings: KeybindingStyle,
    pub ui_mode: UIMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIMode {
    Standard,
    Enhanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub syntax_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub size: u16,
    pub family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub line_numbers: bool,
    pub highlight_current_line: bool,
    pub wrap_lines: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeybindingStyle {
    Nano,
    Vim,
    Emacs,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                name: String::from("dark"),
                syntax_theme: String::from("base16-ocean.dark"),
            },
            font: FontConfig {
                size: 14,
                family: String::from("monospace"),
            },
            editor: EditorConfig {
                tab_size: 4,
                use_spaces: true,
                line_numbers: true,
                highlight_current_line: true,
                wrap_lines: false,
            },
            keybindings: KeybindingStyle::Vim,
            ui_mode: UIMode::Enhanced,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        if let Some(config_path) = Self::config_path() {
            if config_path.exists() {
                let content = tokio::fs::read_to_string(&config_path).await?;
                // Try to deserialize, fall back to default if it fails
                match serde_json::from_str::<Config>(&content) {
                    Ok(config) => return Ok(config),
                    Err(_) => {
                        // If deserialization fails, use default and save it
                        let default_config = Self::default();
                        let _ = default_config.save().await;
                        return Ok(default_config);
                    }
                }
            }
        }
        
        let default_config = Self::default();
        let _ = default_config.save().await;
        Ok(default_config)
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::config_path() {
            if let Some(parent) = config_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let content = serde_json::to_string_pretty(self)?;
            tokio::fs::write(&config_path, content).await?;
        }
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "scriptoris", "scriptoris").map(|dirs| {
            dirs.config_dir().join("config.json")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.theme.name, "dark");
        assert_eq!(config.font.size, 14);
        assert_eq!(config.font.family, "monospace");
        assert_eq!(config.editor.tab_size, 4);
        assert!(config.editor.use_spaces);
        assert!(config.editor.line_numbers);
        assert!(config.editor.highlight_current_line);
        assert!(!config.editor.wrap_lines);
        assert!(matches!(config.keybindings, KeybindingStyle::Vim));
    }

    #[tokio::test]
    async fn test_config_serialization() {
        let config = Config::default();
        
        // Test serialization
        let json = serde_json::to_string_pretty(&config);
        assert!(json.is_ok());
        
        let json = json.unwrap();
        assert!(json.contains("\"theme\""));
        assert!(json.contains("\"font\""));
        assert!(json.contains("\"editor\""));
        assert!(json.contains("\"keybindings\""));
        
        // Test deserialization
        let config_from_json: Result<Config, _> = serde_json::from_str(&json);
        assert!(config_from_json.is_ok());
        
        let config_from_json = config_from_json.unwrap();
        assert_eq!(config.theme.name, config_from_json.theme.name);
        assert_eq!(config.font.size, config_from_json.font.size);
        assert_eq!(config.editor.tab_size, config_from_json.editor.tab_size);
    }

    #[tokio::test]
    async fn test_config_load_default() {
        // This test loads config - if file doesn't exist, should create default
        let config = Config::load().await;
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.theme.name, "dark");
        assert!(matches!(config.keybindings, KeybindingStyle::Vim));
    }

    #[test]
    fn test_keybinding_style_variants() {
        // Test that all keybinding styles can be serialized/deserialized
        let vim = KeybindingStyle::Vim;
        let nano = KeybindingStyle::Nano;
        let emacs = KeybindingStyle::Emacs;
        
        let vim_json = serde_json::to_string(&vim).unwrap();
        let nano_json = serde_json::to_string(&nano).unwrap();
        let emacs_json = serde_json::to_string(&emacs).unwrap();
        
        assert_eq!(vim_json, "\"Vim\"");
        assert_eq!(nano_json, "\"Nano\"");
        assert_eq!(emacs_json, "\"Emacs\"");
        
        // Test deserialization
        let vim_from_json: KeybindingStyle = serde_json::from_str(&vim_json).unwrap();
        assert!(matches!(vim_from_json, KeybindingStyle::Vim));
    }
}