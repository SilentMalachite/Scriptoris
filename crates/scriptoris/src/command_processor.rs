//! `:` で始まるコマンドライン操作を解析し、適切なアクションに変換します。
//! バッファ/ウィンドウ操作は `CommandAction` として呼び出し側に委譲します。

use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::editor::Editor;
use crate::file_manager::FileManager;
use crate::session_manager::SessionManager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    None,
    Buffer(BufferCommand),
    Window(WindowCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferCommand {
    Next,
    Previous,
    List,
    DeleteCurrent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowCommand {
    SplitHorizontal,
    SplitVertical,
}

pub struct CommandProcessor {
    session_manager: SessionManager,
    pending_action: Option<CommandAction>,
}

impl CommandProcessor {
    pub fn new() -> Result<Self> {
        match std::panic::catch_unwind(|| {
            let session_manager = SessionManager::new();
            match session_manager {
                Ok(sm) => Ok(Self {
                    session_manager: sm,
                    pending_action: None,
                }),
                Err(e) => {
                    log::error!("Failed to initialize session manager: {}", e);
                    Err(anyhow::anyhow!(
                        "セッションマネージャーの初期化に失敗しました: {}",
                        e
                    ))
                }
            }
        }) {
            Ok(result) => result,
            Err(e) => {
                log::error!("Panic during command processor initialization: {:?}", e);
                Err(anyhow::anyhow!(
                    "コマンドプロセッサーの初期化中にパニックが発生しました"
                ))
            }
        }
    }

    pub fn take_pending_action(&mut self) -> Option<CommandAction> {
        self.pending_action.take()
    }

    pub async fn execute_command(
        &mut self,
        command: &str,
        editor: &mut Editor,
        file_manager: &mut FileManager,
        config: &mut Config,
        should_quit: &mut bool,
    ) -> Result<String> {
        // Command execution with safe error handling
        self.pending_action = None;
        let cmd = command.trim();

        if cmd.is_empty() {
            return Ok(String::new());
        }

        // Validate command length to prevent potential issues
        if cmd.len() > 1000 {
            return Err(anyhow::anyhow!("コマンドが長すぎます (最大1000文字)"));
        }

        // Check for potentially dangerous commands
        if cmd.contains("..") || cmd.contains('~') && cmd.contains("rm") {
            log::warn!("Potentially dangerous command detected: {}", cmd);
            return Err(anyhow::anyhow!(
                "安全上の理由によりこのコマンドは許可されていません"
            ));
        }

        // Execute command with error handling
        self.execute_command_safe(cmd, editor, file_manager, config, should_quit)
            .await
    }

    async fn execute_command_safe(
        &mut self,
        cmd: &str,
        editor: &mut Editor,
        file_manager: &mut FileManager,
        config: &mut Config,
        should_quit: &mut bool,
    ) -> Result<String> {
        // Handle search commands (starting with search)
        if cmd.starts_with("search ") {
            let query = cmd.strip_prefix("search ").unwrap_or("");
            if !query.is_empty() {
                editor.search(query);
                return Ok(format!("検索: {}", query));
            } else {
                return Err(anyhow::anyhow!("検索文字列が空です"));
            }
        }

        // Handle vim-style commands
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        // Validate number of arguments
        if parts.len() > 10 {
            return Err(anyhow::anyhow!("引数が多すぎます"));
        }

        // Process command parts with error handling
        let result = self
            .process_command_parts(&parts, editor, file_manager, config, should_quit)
            .await;

        // Log command execution result for debugging
        match &result {
            Ok(message) => {
                if !message.is_empty() {
                    log::debug!("Command '{}' executed successfully: {}", cmd, message);
                }
            }
            Err(e) => {
                log::error!("Command '{}' failed: {}", cmd, e);
            }
        }

        result
    }

    async fn process_command_parts(
        &mut self,
        parts: &[&str],
        editor: &mut Editor,
        file_manager: &mut FileManager,
        config: &mut Config,
        should_quit: &mut bool,
    ) -> Result<String> {
        if parts.is_empty() {
            return Ok(String::new());
        }

        match parts[0] {
            "w" => self.handle_save_command(parts, editor, file_manager).await,
            "q" | "q!" => self.handle_quit_command(parts, editor, should_quit),
            "wq" => {
                self.handle_save_quit_command(editor, file_manager, should_quit)
                    .await
            }
            "e" => self.handle_edit_command(parts, file_manager, editor).await,
            "split" | "sp" | "vsplit" | "vsp" | "bnext" | "bn" | "bprev" | "bp" | "buffers"
            | "ls" | "bdelete" | "bd" => self.handle_window_buffer_commands(parts[0]),
            "mksession" => {
                self.handle_session_save_command(parts, editor, file_manager, config)
                    .await
            }
            "source" => {
                self.handle_session_load_command(parts, file_manager, editor, config)
                    .await
            }
            "sessions" => self.handle_session_list_command().await,
            "delsession" => self.handle_session_delete_command(parts).await,
            "set" => self.handle_set_command(parts, config).await,
            _ => Err(anyhow::anyhow!("E492: 未定義のコマンドです: {}", parts[0])),
        }
    }

    async fn handle_save_command(
        &self,
        parts: &[&str],
        editor: &mut Editor,
        file_manager: &mut FileManager,
    ) -> Result<String> {
        if parts.len() > 1 {
            let path = PathBuf::from(parts[1]);
            log::info!("Save as command with path: {:?}", path);
        }

        if parts.len() > 1 {
            // :w filename - save as
            let path = PathBuf::from(parts[1]);
            match file_manager.save_file_as(path, editor).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    log::error!("Save as failed: {}", e);
                    Err(anyhow::anyhow!("名前を付けて保存に失敗しました: {}", e))
                }
            }
        } else if file_manager.has_file() {
            match file_manager.save_file(editor).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    log::error!("Save failed: {}", e);
                    Err(anyhow::anyhow!("保存に失敗しました: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!(
                "ファイル名が指定されていません (:w <filename> で名前を付けて保存)"
            ))
        }
    }

    fn handle_quit_command(
        &self,
        parts: &[&str],
        editor: &Editor,
        should_quit: &mut bool,
    ) -> Result<String> {
        if parts[0] == "q!" {
            // :q! - force quit
            *should_quit = true;
            return Ok("強制終了します".to_string());
        }

        // :q - quit
        if editor.is_modified() {
            Ok("変更が保存されていません (:q! で強制終了)".to_string())
        } else {
            *should_quit = true;
            Ok("終了します".to_string())
        }
    }

    async fn handle_save_quit_command(
        &self,
        editor: &mut Editor,
        file_manager: &mut FileManager,
        should_quit: &mut bool,
    ) -> Result<String> {
        let result = if file_manager.has_file() {
            match file_manager.save_file(editor).await {
                Ok(result) => result,
                Err(e) => {
                    log::error!("Save in wq failed: {}", e);
                    return Err(anyhow::anyhow!("保存に失敗したため終了できません: {}", e));
                }
            }
        } else {
            return Err(anyhow::anyhow!("ファイル名が指定されていません"));
        };
        *should_quit = true;
        Ok(format!("{}。エディタを終了します", result))
    }

    async fn handle_edit_command(
        &self,
        parts: &[&str],
        file_manager: &mut FileManager,
        editor: &mut Editor,
    ) -> Result<String> {
        if parts.len() <= 1 {
            return Err(anyhow::anyhow!("E471: 引数が必要です (:e <filename>)"));
        }

        let path = PathBuf::from(parts[1]);
        log::info!("Edit command with path: {:?}", path);

        match file_manager.open_file(path).await {
            Ok(content) => {
                editor.set_content(content);
                Ok("ファイルを開きました".to_string())
            }
            Err(e) => {
                log::error!("File open failed: {}", e);
                Err(anyhow::anyhow!("ファイルを開けませんでした: {}", e))
            }
        }
    }

    fn handle_window_buffer_commands(&mut self, command: &str) -> Result<String> {
        self.pending_action = Some(match command {
            "split" | "sp" => CommandAction::Window(WindowCommand::SplitHorizontal),
            "vsplit" | "vsp" => CommandAction::Window(WindowCommand::SplitVertical),
            "bnext" | "bn" => CommandAction::Buffer(BufferCommand::Next),
            "bprev" | "bp" => CommandAction::Buffer(BufferCommand::Previous),
            "buffers" | "ls" => CommandAction::Buffer(BufferCommand::List),
            "bdelete" | "bd" => CommandAction::Buffer(BufferCommand::DeleteCurrent),
            _ => CommandAction::None,
        });
        Ok(String::new())
    }

    async fn handle_session_save_command(
        &mut self,
        parts: &[&str],
        editor: &Editor,
        file_manager: &FileManager,
        config: &Config,
    ) -> Result<String> {
        if parts.len() <= 1 {
            return Err(anyhow::anyhow!("使い方: :mksession <名前>"));
        }

        let session_name = parts[1];
        log::info!("Saving session: {}", session_name);

        match self
            .session_manager
            .save_session(session_name, editor, file_manager, config)
            .await
        {
            Ok(_) => Ok(format!("セッション '{}' を保存しました", session_name)),
            Err(e) => {
                log::error!("Session save failed: {}", e);
                Err(anyhow::anyhow!("セッションの保存に失敗しました: {}", e))
            }
        }
    }

    async fn handle_session_load_command(
        &mut self,
        parts: &[&str],
        file_manager: &mut FileManager,
        editor: &mut Editor,
        config: &mut Config,
    ) -> Result<String> {
        if parts.len() <= 1 {
            return Err(anyhow::anyhow!("使い方: :source <セッション名>"));
        }

        let session_name = parts[1];
        log::info!("Loading session: {}", session_name);

        match self.session_manager.load_session(session_name).await {
            Ok(session_data) => {
                // Restore file first (this sets the content)
                if let Some(file_path) = &session_data.current_file {
                    match file_manager.open_file(file_path.clone()).await {
                        Ok(content) => {
                            editor.set_content(content);
                        }
                        Err(e) => {
                            log::error!("Failed to open session file: {}", e);
                            return Err(anyhow::anyhow!(
                                "セッションのファイルを開けませんでした: {}",
                                e
                            ));
                        }
                    }
                } else {
                    // No file in session, just set empty content
                    editor.set_content(String::new());
                }

                // Restore other session data
                editor.set_cursor_position(session_data.cursor_line, session_data.cursor_col);
                editor.set_viewport_offset(session_data.viewport_offset);

                // Restore editor config
                config.editor.tab_size = session_data.editor_config.tab_size;
                config.editor.use_spaces = session_data.editor_config.use_spaces;
                config.editor.line_numbers = session_data.editor_config.line_numbers;
                config.editor.highlight_current_line =
                    session_data.editor_config.highlight_current_line;
                config.editor.wrap_lines = session_data.editor_config.wrap_lines;

                // Apply tab config to editor
                editor.set_tab_config(config.editor.tab_size, config.editor.use_spaces);

                Ok(format!("セッション '{}' を読み込みました", session_name))
            }
            Err(e) => {
                log::error!("Session load failed: {}", e);
                Err(anyhow::anyhow!("セッションの読み込みに失敗しました: {}", e))
            }
        }
    }

    async fn handle_session_list_command(&self) -> Result<String> {
        match self.session_manager.list_sessions().await {
            Ok(sessions) => {
                if sessions.is_empty() {
                    Ok("セッションは見つかりません".to_string())
                } else {
                    let mut result = String::from("利用可能なセッション:\n");
                    for session in sessions {
                        result.push_str(&format!(
                            "  {} (更新日時: {})\n",
                            session.name,
                            session.modified_at.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                    Ok(result.trim_end().to_string())
                }
            }
            Err(e) => {
                log::error!("Session list failed: {}", e);
                Err(anyhow::anyhow!("セッション一覧の取得に失敗しました: {}", e))
            }
        }
    }

    async fn handle_session_delete_command(&mut self, parts: &[&str]) -> Result<String> {
        if parts.len() <= 1 {
            return Err(anyhow::anyhow!("使い方: :delsession <セッション名>"));
        }

        let session_name = parts[1];
        log::info!("Deleting session: {}", session_name);

        match self.session_manager.delete_session(session_name).await {
            Ok(_) => Ok(format!("セッション '{}' を削除しました", session_name)),
            Err(e) => {
                log::error!("Session delete failed: {}", e);
                Err(anyhow::anyhow!("セッションの削除に失敗しました: {}", e))
            }
        }
    }

    async fn handle_set_command(&self, parts: &[&str], config: &mut Config) -> Result<String> {
        if parts.len() != 3 || parts[1] != "theme" {
            return Err(anyhow::anyhow!("使い方: :set theme <テーマ名>"));
        }

        let theme = parts[2];
        log::info!("Setting theme to: {}", theme);

        // Validate theme name
        if theme.is_empty() || theme.len() > 100 {
            return Err(anyhow::anyhow!("無効なテーマ名です"));
        }

        config.theme.syntax_theme = theme.to_string();

        // Persist configuration with error handling
        match config.save().await {
            Ok(_) => {
                log::info!("Configuration saved with new theme: {}", theme);
                Ok(format!("テーマを '{}' に設定しました", theme))
            }
            Err(e) => {
                log::error!("Failed to save config after theme change: {}", e);
                Ok(format!(
                    "テーマを '{}' に設定しました (設定の保存に失敗)",
                    theme
                ))
            }
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
        assert!(CommandProcessor::new().is_ok());
    }

    #[tokio::test]
    async fn test_quit_commands() {
        let mut editor = Editor::new();
        let config = Config::default();
        editor.set_tab_config(config.editor.tab_size, config.editor.use_spaces);
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        // Test quit with unmodified editor
        let mut config = Config::default();
        let mut cp = CommandProcessor::new().expect("command processor should initialize");
        let result = cp
            .execute_command(
                "q",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
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
        if let Err(e) = &result {
            eprintln!("Second q command error: {}", e);
        }
        let message = result.expect("result should contain warning");
        assert!(!should_quit); // Should not quit due to modifications
        assert!(message.contains("変更が保存されていません"));

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
        if let Err(e) = &result {
            eprintln!("Force quit error: {}", e);
        }
        assert!(result.is_ok(), "q! command failed: {:?}", result);
        assert!(should_quit); // Should force quit
    }

    #[tokio::test]
    async fn test_search_command() {
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        editor.set_content("Hello World\nTest line".to_string());

        let mut config = Config::default();
        let mut cp = CommandProcessor::new().expect("command processor should initialize");
        let result = cp
            .execute_command(
                "search World",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;
        let message = result.expect("result should contain search status");
        assert!(message.contains("検索: World"));

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
        let mut cp = CommandProcessor::new().expect("command processor should initialize");
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

    #[tokio::test]
    async fn test_buffer_action_emits_pending_action() {
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut config = Config::default();
        let mut should_quit = false;
        let mut processor = CommandProcessor::new().expect("command processor should initialize");

        let result = processor
            .execute_command(
                "bn",
                &mut editor,
                &mut file_manager,
                &mut config,
                &mut should_quit,
            )
            .await;

        let message = result.expect("command should succeed");
        assert!(message.is_empty());
        match processor.take_pending_action() {
            Some(CommandAction::Buffer(BufferCommand::Next)) => {}
            other => panic!("unexpected action: {:?}", other),
        }
    }
}
