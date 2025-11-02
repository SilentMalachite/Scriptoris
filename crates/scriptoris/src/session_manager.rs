//! エディタのセッション（開いているファイルやカーソル位置など）を
//! JSON 形式で保存・復元するモジュール。

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::try_exists;

use crate::config::Config;
use crate::editor::Editor;
use crate::file_manager::FileManager;

/// Session data that can be saved and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub current_file: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub viewport_offset: usize,
    pub editor_config: EditorConfigSnapshot,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfigSnapshot {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub line_numbers: bool,
    pub highlight_current_line: bool,
    pub wrap_lines: bool,
}

/// Session manager for saving and loading editor sessions
pub struct SessionManager {
    session_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let session_dir = Self::get_session_dir()?;
        Ok(Self { session_dir })
    }

    fn get_session_dir() -> Result<PathBuf> {
        if let Ok(dir) = std::env::var("SCRIPTORIS_DATA_DIR") {
            return Ok(PathBuf::from(dir).join("sessions"));
        }
        let dirs = directories::ProjectDirs::from("com", "scriptoris", "scriptoris")
            .ok_or_else(|| anyhow::anyhow!("プロジェクトディレクトリを特定できませんでした"))?;
        let session_dir = dirs.data_dir().join("sessions");
        Ok(session_dir)
    }

    /// Create session directory if it doesn't exist
    async fn ensure_session_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.session_dir).await?;
        Ok(())
    }

    /// Save current session with given name
    pub async fn save_session(
        &self,
        name: &str,
        editor: &Editor,
        file_manager: &FileManager,
        config: &Config,
    ) -> Result<String> {
        self.ensure_session_dir().await?;

        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(&filename);

        let existing_created_at = if try_exists(&filepath).await? {
            match fs::read_to_string(&filepath).await {
                Ok(json) => match serde_json::from_str::<SessionData>(&json) {
                    Ok(session) => Some(session.created_at),
                    Err(e) => {
                        log::warn!("Failed to parse existing session '{}': {}", name, e);
                        None
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Failed to read existing session file '{}': {}",
                        filepath.display(),
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        let now = Utc::now();
        let session_data = SessionData {
            name: name.to_string(),
            created_at: existing_created_at.unwrap_or(now),
            modified_at: now,
            current_file: file_manager.get_current_path().cloned(),
            cursor_line: editor.cursor_position().0,
            cursor_col: editor.cursor_position().1,
            viewport_offset: editor.get_viewport_offset(),
            editor_config: EditorConfigSnapshot {
                tab_size: config.editor.tab_size,
                use_spaces: config.editor.use_spaces,
                line_numbers: config.editor.line_numbers,
                highlight_current_line: config.editor.highlight_current_line,
                wrap_lines: config.editor.wrap_lines,
            },
            readonly: file_manager.is_readonly(),
        };
        let json = serde_json::to_string_pretty(&session_data)?;
        fs::write(&filepath, json).await?;

        Ok(format!("セッション '{}' を保存しました", name))
    }

    /// Load session by name
    pub async fn load_session(&self, name: &str) -> Result<SessionData> {
        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(filename);

        if !try_exists(&filepath).await? {
            return Err(anyhow::anyhow!("セッション '{}' は見つかりません", name));
        }

        let json = fs::read_to_string(&filepath).await?;
        let mut session_data: SessionData = serde_json::from_str(&json)?;

        // Update modified timestamp
        session_data.modified_at = Utc::now();
        let updated_json = serde_json::to_string_pretty(&session_data)?;
        fs::write(&filepath, updated_json).await?;

        Ok(session_data)
    }

    /// List all available sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionData>> {
        if !try_exists(&self.session_dir).await? {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut dir_entries = fs::read_dir(&self.session_dir).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path).await {
                    if let Ok(session) = serde_json::from_str::<SessionData>(&json) {
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by modified time (most recent first)
        sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        Ok(sessions)
    }

    /// Delete session by name
    pub async fn delete_session(&self, name: &str) -> Result<String> {
        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(filename);

        if !try_exists(&filepath).await? {
            return Err(anyhow::anyhow!("セッション '{}' は見つかりません", name));
        }

        fs::remove_file(&filepath).await?;
        Ok(format!("セッション '{}' を削除しました", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::{NamedTempFile, TempDir};

    fn session_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_data_dir(path: &std::path::Path) -> Option<String> {
        let previous = std::env::var("SCRIPTORIS_DATA_DIR").ok();
        std::env::set_var("SCRIPTORIS_DATA_DIR", path);
        previous
    }

    fn restore_data_dir(previous: Option<String>) {
        if let Some(value) = previous {
            std::env::set_var("SCRIPTORIS_DATA_DIR", value);
        } else {
            std::env::remove_var("SCRIPTORIS_DATA_DIR");
        }
    }

    #[tokio::test]
    async fn test_save_load_and_delete_session() {
        let previous_env = {
            let _guard = session_test_lock().lock().unwrap();
            let data_dir = TempDir::new().unwrap();
            set_data_dir(data_dir.path())
        }; // release lock before await

        let manager = SessionManager::new().expect("session manager should initialize");

        let mut editor = Editor::new();
        editor.set_content("Hello".to_string());
        editor.set_cursor_position(0, 5);
        editor.set_viewport_offset(0);

        let mut file_manager = FileManager::new();
        let temp_file = NamedTempFile::new().unwrap();
        file_manager.current_path = Some(temp_file.path().to_path_buf());

        let mut config = Config::default();
        config.editor.tab_size = 2;
        config.editor.use_spaces = false;

        let save_message = manager
            .save_session("test", &editor, &file_manager, &config)
            .await
            .expect("session should save");
        assert!(save_message.contains("保存"));

        let loaded = manager
            .load_session("test")
            .await
            .expect("session should load");
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.cursor_col, 4);
        assert_eq!(loaded.editor_config.tab_size, 2);

        let delete_message = manager
            .delete_session("test")
            .await
            .expect("session should delete");
        assert!(delete_message.contains("削除"));

        restore_data_dir(previous_env);
    }

    #[tokio::test]
    async fn test_load_missing_session_returns_error() {
        let previous_env = {
            let _guard = session_test_lock().lock().unwrap();
            let data_dir = TempDir::new().unwrap();
            set_data_dir(data_dir.path())
        }; // release lock before await

        let manager = SessionManager::new().expect("session manager should initialize");
        let error = manager.load_session("missing").await.unwrap_err();
        assert!(error.to_string().contains("見つかりません"));

        restore_data_dir(previous_env);
    }
}
