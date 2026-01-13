use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::try_exists;

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
    #[serde(default)]
    pub editor_foreground: Option<String>,
    #[serde(default)]
    pub editor_background: Option<String>,
    #[serde(default)]
    pub accent_color: Option<String>,
    #[serde(default)]
    pub status_background: Option<String>,
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
                editor_foreground: Some(String::from("#D8DEE9")),
                editor_background: Some(String::from("#1E1E1E")),
                accent_color: Some(String::from("#FFD166")),
                status_background: Some(String::from("#005F87")),
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
            if try_exists(&config_path).await? {
                // Try to read existing config
                match tokio::fs::read_to_string(&config_path).await {
                    Ok(content) => {
                        // Validate JSON format
                        if content.trim().is_empty() {
                            log::warn!("Config file is empty, creating new one");
                            let default_config = Self::default();
                            let _ = default_config.save().await;
                            return Ok(default_config);
                        }

                        // Try to deserialize
                        match serde_json::from_str::<Self>(&content) {
                            Ok(mut config) => {
                                // Validate config values
                                config.validate()?;
                                log::info!(
                                    "Successfully loaded config from: {}",
                                    config_path.display()
                                );
                                return Ok(config);
                            }
                            Err(json_err) => {
                                log::error!("Failed to parse config file: {}", json_err);

                                // Backup broken config
                                let backup_path = config_path.with_extension("bak");
                                if let Err(e) = tokio::fs::copy(&config_path, &backup_path).await {
                                    log::warn!("Failed to backup broken config: {}", e);
                                } else {
                                    log::info!(
                                        "Backed up broken config to: {}",
                                        backup_path.display()
                                    );
                                }

                                // Use default config
                                let default_config = Self::default();
                                let _ = default_config.save().await;
                                return Ok(default_config);
                            }
                        }
                    }
                    Err(io_err) => {
                        log::error!("Failed to read config file: {}", io_err);
                    }
                }
            } else {
                log::info!("Config file does not exist, creating default");
            }
        }

        // Create default config
        let default_config = Self::default();
        let _ = default_config.save().await;
        Ok(default_config)
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::config_path() {
            // Validate before saving
            let mut config_to_save = self.clone();
            config_to_save.validate()?;

            // Create config directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                match tokio::fs::create_dir_all(parent).await {
                    Ok(_) => {
                        log::debug!(
                            "Config directory exists or was created: {}",
                            parent.display()
                        );
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "設定ディレクトリの作成に失敗しました: {} - {}",
                            parent.display(),
                            e
                        ));
                    }
                }
            }

            // Serialize and save with error handling
            match serde_json::to_string_pretty(&config_to_save) {
                Ok(content) => match tokio::fs::write(&config_path, content).await {
                    Ok(_) => {
                        log::info!("Successfully saved config to: {}", config_path.display());
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "設定ファイルの書き込みに失敗しました: {} - {}",
                            config_path.display(),
                            e
                        ));
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("設定のシリアライズに失敗しました: {}", e));
                }
            }
        }
        Ok(())
    }

    /// Validate configuration values and fix invalid ones
    pub fn validate(&mut self) -> Result<()> {
        let mut has_issues = false;

        // Validate font size
        if self.font.size < 6 || self.font.size > 72 {
            log::warn!("Invalid font size: {}, using default", self.font.size);
            self.font.size = 14;
            has_issues = true;
        }

        // Validate tab size
        if self.editor.tab_size == 0 || self.editor.tab_size > 16 {
            log::warn!("Invalid tab size: {}, using default", self.editor.tab_size);
            self.editor.tab_size = 4;
            has_issues = true;
        }

        // Validate theme name
        if self.theme.name.is_empty() {
            log::warn!("Empty theme name, using default");
            self.theme.name = "dark".to_string();
            has_issues = true;
        }

        // Validate syntax theme
        if self.theme.syntax_theme.is_empty() {
            log::warn!("Empty syntax theme, using default");
            self.theme.syntax_theme = "base16-ocean.dark".to_string();
            has_issues = true;
        }

        if has_issues {
            log::info!("Configuration validation completed with corrections");
        }

        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("SCRIPTORIS_CONFIG_PATH") {
            return Some(PathBuf::from(path));
        }

        if let Ok(dir) = std::env::var("SCRIPTORIS_CONFIG_DIR") {
            return Some(PathBuf::from(dir).join("config.json"));
        }

        ProjectDirs::from("com", "scriptoris", "scriptoris")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn config_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_config_dir(path: &std::path::Path) -> (Option<String>, Option<String>) {
        let previous_dir = std::env::var("SCRIPTORIS_CONFIG_DIR").ok();
        let previous_path = std::env::var("SCRIPTORIS_CONFIG_PATH").ok();
        std::env::set_var("SCRIPTORIS_CONFIG_DIR", path);
        std::env::remove_var("SCRIPTORIS_CONFIG_PATH");
        (previous_dir, previous_path)
    }

    fn restore_config_env(previous: (Option<String>, Option<String>)) {
        match previous.0 {
            Some(value) => std::env::set_var("SCRIPTORIS_CONFIG_DIR", value),
            None => std::env::remove_var("SCRIPTORIS_CONFIG_DIR"),
        }

        match previous.1 {
            Some(value) => std::env::set_var("SCRIPTORIS_CONFIG_PATH", value),
            None => std::env::remove_var("SCRIPTORIS_CONFIG_PATH"),
        }
    }

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
        assert_eq!(config.theme.editor_foreground.as_deref(), Some("#D8DEE9"));
        assert_eq!(config.theme.editor_background.as_deref(), Some("#1E1E1E"));
        assert_eq!(config.theme.accent_color.as_deref(), Some("#FFD166"));
        assert_eq!(config.theme.status_background.as_deref(), Some("#005F87"));
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
        assert_eq!(
            config.theme.editor_foreground,
            config_from_json.theme.editor_foreground
        );
    }

    #[tokio::test]
    async fn test_config_load_default() {
        // Serialize/deserialize in isolated directory to avoid touching user config
        let previous_env = {
            let _guard = config_test_lock().lock().unwrap();
            let temp_dir = TempDir::new().unwrap();
            let previous = set_config_dir(temp_dir.path());
            previous
        }; // release lock before await

        let config = Config::load().await;
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.theme.name, "dark");
        assert!(matches!(config.keybindings, KeybindingStyle::Vim));

        restore_config_env(previous_env);
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
