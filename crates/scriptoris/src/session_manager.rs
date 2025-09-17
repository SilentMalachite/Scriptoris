use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use chrono::{DateTime, Utc};

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
        let dirs = directories::ProjectDirs::from("com", "scriptoris", "scriptoris")
            .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))?;
        let session_dir = dirs.data_dir().join("sessions");
        Ok(session_dir)
    }

    /// Create session directory if it doesn't exist
    async fn ensure_session_dir(&self) -> Result<()> {
        if !self.session_dir.exists() {
            fs::create_dir_all(&self.session_dir).await?;
        }
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

        let session_data = SessionData {
            name: name.to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
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

        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(filename);
        let json = serde_json::to_string_pretty(&session_data)?;
        fs::write(&filepath, json).await?;

        Ok(format!("Session '{}' saved", name))
    }

    /// Load session by name
    pub async fn load_session(&self, name: &str) -> Result<SessionData> {
        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(filename);

        if !filepath.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", name));
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
        if !self.session_dir.exists() {
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

        if !filepath.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", name));
        }

        fs::remove_file(&filepath).await?;
        Ok(format!("Session '{}' deleted", name))
    }

    /// Check if session exists
    pub async fn session_exists(&self, name: &str) -> bool {
        let filename = format!("{}.json", name);
        let filepath = self.session_dir.join(filename);
        filepath.exists()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
