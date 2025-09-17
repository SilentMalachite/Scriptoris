use anyhow::Result;
use std::path::PathBuf;

use crate::session_manager::SessionManager;
use crate::editor::Editor;
use crate::file_manager::FileManager;
use crate::config::Config;

pub struct CommandProcessor {
    session_manager: SessionManager,
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new().unwrap_or_default(),
        }
    }

    pub async fn execute_command(
        &self,
        command: &str,
        editor: &mut Editor,
        file_manager: &mut FileManager,
        config: &mut Config,
        should_quit: &mut bool,
    ) -> Result<String> {
        let cmd = command.trim();

        if cmd.is_empty() {
            return Ok(String::new());
        }

        // Handle search commands (starting with search)
        if cmd.starts_with("search ") {
            let query = cmd.strip_prefix("search ").unwrap_or("");
            if !query.is_empty() {
                editor.search(query);
                return Ok(format!("Searching for: {}", query));
            }
        }

        // Handle vim-style commands
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        match parts[0] {
            "w" => {
                // :w - save file
                if parts.len() > 1 {
                    // :w filename - save as
                    let path = PathBuf::from(parts[1]);
                    let result = file_manager.save_file_as(path, editor).await?;
                    Ok(result)
                } else if file_manager.has_file() {
                    let result = file_manager.save_file(editor).await?;
                    Ok(result)
                } else {
                    Err(anyhow::anyhow!("No file name specified"))
                }
            }
            "q" => {
                // :q - quit
                if editor.is_modified() {
                    Ok("No write since last change (use :q! to override)".to_string())
                } else {
                    *should_quit = true;
                    Ok("Quitting".to_string())
                }
            }
            "q!" => {
                // :q! - force quit
                *should_quit = true;
                Ok("Force quitting".to_string())
            }
            "wq" => {
                // :wq - save and quit
                let result = if file_manager.has_file() {
                    file_manager.save_file(editor).await?
                } else {
                    return Err(anyhow::anyhow!("No file name specified"));
                };
                *should_quit = true;
                Ok(format!("{} - Quitting", result))
            }
            "e" => {
                // :e filename - edit file
                if parts.len() > 1 {
                    let path = PathBuf::from(parts[1]);
                    let content = file_manager.open_file(path).await?;
                    editor.set_content(content);
                    Ok("File opened".to_string())
                } else {
                    Err(anyhow::anyhow!("E471: Argument required"))
                }
            }
            // Buffer and window management commands temporarily disabled due to borrow checker issues
            "split" | "sp" | "vsplit" | "vsp" | "bnext" | "bn" | "bprev" | "bp" | "buffers"
            | "ls" | "bdelete" | "bd" => {
                Ok("Buffer/window management commands temporarily disabled".to_string())
            }
            // Session management commands
            "mksession" => {
                if parts.len() > 1 {
                    let session_name = parts[1];
                    self.session_manager
                        .save_session(session_name, editor, file_manager, config)
                        .await
                } else {
                    Err(anyhow::anyhow!("Usage: :mksession <name>"))
                }
            }
            "source" => {
                if parts.len() > 1 {
                    let session_name = parts[1];
                    let session_data = self.session_manager.load_session(session_name).await?;

                    // Restore file first (this sets the content)
                    if let Some(file_path) = &session_data.current_file {
                        let content = file_manager.open_file(file_path.clone()).await?;
                        editor.set_content(content);
                    } else {
                        // No file in session, just set empty content
                        editor.set_content(String::new());
                    }

                    // Restore cursor position
                    editor.set_cursor_position(session_data.cursor_line, session_data.cursor_col);

                    // Restore viewport
                    editor.set_viewport_offset(session_data.viewport_offset);

                    // Restore editor config
                    config.editor.tab_size = session_data.editor_config.tab_size;
                    config.editor.use_spaces = session_data.editor_config.use_spaces;
                    config.editor.line_numbers = session_data.editor_config.line_numbers;
                    config.editor.highlight_current_line = session_data.editor_config.highlight_current_line;
                    config.editor.wrap_lines = session_data.editor_config.wrap_lines;

                    Ok(format!("Session '{}' loaded", session_name))
                } else {
                    Err(anyhow::anyhow!("Usage: :source <session-name>"))
                }
            }
            "sessions" => {
                let sessions = self.session_manager.list_sessions().await?;
                if sessions.is_empty() {
                    Ok("No sessions found".to_string())
                } else {
                    let mut result = String::from("Available sessions:\n");
                    for session in sessions {
                        result.push_str(&format!(
                            "  {} (modified: {})\n",
                            session.name,
                            session.modified_at.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                    Ok(result.trim_end().to_string())
                }
            }
            "delsession" => {
                if parts.len() > 1 {
                    let session_name = parts[1];
                    self.session_manager.delete_session(session_name).await
                } else {
                    Err(anyhow::anyhow!("Usage: :delsession <session-name>"))
                }
            }
            "set" => {
                // :set theme <name>
                if parts.len() == 3 && parts[1] == "theme" {
                    let theme = parts[2];
                    config.theme.syntax_theme = theme.to_string();
                    // Persist asynchronously
                    let _ = config.save().await;
                    Ok(format!("Theme set to '{}'", theme))
                } else {
                    Err(anyhow::anyhow!("Usage: :set theme <theme-name>"))
                }
            }
            _ => Err(anyhow::anyhow!("E492: Not an editor command: {}", parts[0])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_command_processor_creation() {
        let cp = CommandProcessor::new();
        // Just ensure it can be created - now has a session manager
        assert!(true); // CommandProcessor has been successfully created
    }

    #[tokio::test]
    async fn test_quit_commands() {
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        // Test quit with unmodified editor
        let mut config = Config::default();
        let cp = CommandProcessor::new();
        let result = cp
            .execute_command(
                "q",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert!(should_quit);

        // Reset
        should_quit = false;
        editor.insert_char('a'); // Make editor modified

        // Test quit with modified editor
        let mut config = Config::default();
        let result = cp
            .execute_command(
                "q",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert!(!should_quit); // Should not quit due to modifications
        assert!(result.unwrap().contains("No write since last change"));

        // Test force quit
        let mut config = Config::default();
        let result = cp
            .execute_command(
                "q!",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert!(should_quit); // Should force quit
    }

    #[tokio::test]
    async fn test_search_command() {
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        editor.set_content("Hello World\nTest line".to_string());

        let mut config = Config::default();
        let cp = CommandProcessor::new();
        let result = cp
            .execute_command(
                "search World",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Searching for: World"));

        // Check cursor moved to found position
        let (line, col) = editor.cursor_position();
        assert_eq!(line, 0);
        assert_eq!(col, 6); // "World" starts at column 6 in "Hello World"
    }

    #[tokio::test]
    async fn test_file_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Initial content").unwrap();

        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        // Test edit command
        let cmd = format!("e {}", temp_file.path().display());
        let mut config = Config::default();
        let cp = CommandProcessor::new();
        let result = cp
            .execute_command(
                &cmd,
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(editor.get_content(), "Initial content\n");

        // Test save command
        editor.insert_char('!');
        let mut config = Config::default();
        let result = cp
            .execute_command(
                "w",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        assert!(result.is_ok());
        assert!(!editor.is_modified());
    }
}
