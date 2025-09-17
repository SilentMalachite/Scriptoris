use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::command_processor::CommandProcessor;
use crate::config::Config;
use crate::editor::Editor;
use crate::file_manager::FileManager;
use crate::ui_state::UIState;

#[derive(Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,      // Visual selection mode
    VisualBlock, // Visual block (rectangular) selection mode
    Replace,     // Replace mode
    Help,
    SavePrompt,
}

pub struct App {
    pub config: Config,
    pub ui_state: UIState,
    pub file_manager: FileManager,
    pub command_processor: CommandProcessor,
    pub buffer_manager: BufferManager,
    pub window_manager: WindowManager,
    highlighter_cache: Option<crate::highlight::Highlighter>, // Cache highlighter
    last_key: Option<char>, // For handling multi-key commands like dd
    // Macro recording
    macro_recording: bool,
    macro_register: Option<char>,
    macro_keys: Vec<KeyEvent>,
    macro_registers: std::collections::HashMap<char, Vec<KeyEvent>>,
}

// バッファ管理
pub struct Buffer {
    pub id: usize,
    pub content: Editor,
    pub file_path: Option<std::path::PathBuf>,
    pub readonly: bool,
}

impl Buffer {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            content: Editor::default(),
            file_path: None,
            readonly: false,
        }
    }
}

pub struct BufferManager {
    pub buffers: Vec<Buffer>,
    current_buffer: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        let mut manager = Self {
            buffers: Vec::new(),
            current_buffer: 0,
        };
        // デフォルトバッファを作成
        manager.buffers.push(Buffer::new(0));
        manager
    }

    pub fn get_current(&self) -> &Buffer {
        &self.buffers[self.current_buffer]
    }

    pub fn get_current_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current_buffer]
    }
}

// ウィンドウ管理
#[derive(Clone, Debug)]
pub enum Split {
    Leaf { buffer_id: usize },
}

#[derive(Clone, Debug)]
pub struct Window {
    pub id: usize,
    pub split: Split,
}

impl Window {
    pub fn new_leaf(id: usize, buffer_id: usize) -> Self {
        Self {
            id,
            split: Split::Leaf { buffer_id },
        }
    }
}

pub struct WindowManager {
    root: Window,
    pub current_window_id: usize,
}

// プラグインシステム
#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, app: &mut App) -> Result<()>;
    fn on_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool>;
    fn on_command(&mut self, app: &mut App, command: &str) -> Result<Option<String>>;
    fn on_save(&mut self, app: &mut App, path: &std::path::Path) -> Result<()>;
}

#[allow(dead_code)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

#[allow(dead_code)]
impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn load_plugin(&mut self, mut plugin: Box<dyn Plugin>, app: &mut App) -> Result<()> {
        plugin.on_load(app)?;
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool> {
        for plugin in &mut self.plugins {
            if plugin.on_key(app, key)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn handle_command(&mut self, app: &mut App, command: &str) -> Result<Option<String>> {
        for plugin in &mut self.plugins {
            if let Some(result) = plugin.on_command(app, command)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    pub fn handle_save(&mut self, app: &mut App, path: &std::path::Path) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.on_save(app, path)?;
        }
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins
            .iter()
            .map(|p| (p.name(), p.version()))
            .collect()
    }
}

impl WindowManager {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            root: Window::new_leaf(0, buffer_id),
            current_window_id: 0,
        }
    }

    pub fn get_root(&self) -> &Window {
        &self.root
    }
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::load().await?;
        let buffer_manager = BufferManager::new();
        let initial_buffer_id = buffer_manager.get_current().id;
        Ok(Self {
            config,
            ui_state: UIState::new(),
            file_manager: FileManager::new(),
            command_processor: CommandProcessor::new(),
            buffer_manager,
            window_manager: WindowManager::new(initial_buffer_id),
            highlighter_cache: None,
            last_key: None,
            macro_recording: false,
            macro_register: None,
            macro_keys: Vec::new(),
            macro_registers: std::collections::HashMap::new(),
        })
    }

    pub fn is_modified(&self) -> bool {
        self.buffer_manager.get_current().content.is_modified()
    }

    pub fn get_current_editor(&self) -> &Editor {
        &self.buffer_manager.get_current().content
    }

    pub fn get_current_editor_mut(&mut self) -> &mut Editor {
        &mut self.buffer_manager.get_current_mut().content
    }

    pub fn get_highlighter(&mut self) -> &crate::highlight::Highlighter {
        if self.highlighter_cache.is_none()
            || self.highlighter_cache.as_ref().unwrap().theme_name()
                != &self.config.theme.syntax_theme
        {
            self.highlighter_cache = Some(crate::highlight::Highlighter::new(
                &self.config.theme.syntax_theme,
            ));
        }
        self.highlighter_cache.as_ref().unwrap()
    }

    // Public getters for UI and main.rs
    pub fn should_quit(&self) -> bool {
        self.ui_state.should_quit()
    }

    // Legacy properties for backward compatibility
    pub fn show_help(&self) -> bool {
        self.ui_state.is_help_shown()
    }

    pub fn file_path(&self) -> Option<&std::path::PathBuf> {
        self.file_manager.get_current_path()
    }

    pub fn mode(&self) -> &Mode {
        self.ui_state.get_mode()
    }

    pub fn status_message(&self) -> &str {
        self.ui_state.get_status_message()
    }

    pub fn command_buffer(&self) -> &str {
        self.ui_state.get_command_buffer()
    }

    pub fn quit(&mut self) {
        self.ui_state.quit();
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.ui_state.get_mode() {
            Mode::Normal | Mode::Insert | Mode::Visual | Mode::VisualBlock | Mode::Replace => {
                self.handle_editor_key(key).await?
            }
            Mode::Command => self.handle_command_key(key).await?,
            Mode::Help => self.handle_help_key(key)?,
            Mode::SavePrompt => self.handle_save_prompt_key(key).await?,
        }
        Ok(())
    }

    async fn handle_editor_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.ui_state.get_mode() {
            Mode::Normal => self.handle_normal_mode_key(key),
            Mode::Insert => self.handle_insert_mode_key(key),
            Mode::Visual | Mode::VisualBlock => self.handle_visual_mode_key(key),
            Mode::Replace => self.handle_replace_mode_key(key),
            _ => Ok(()), // Other modes handled elsewhere
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        // Record macro if recording
        if self.macro_recording && key.code != KeyCode::Char('q') {
            self.macro_keys.push(key.clone());
        }

        // Clear last_key if it's not 'd' and we're not pressing 'd'
        if key.code != KeyCode::Char('d')
            && key.code != KeyCode::Char('q')
            && self.last_key.is_some()
        {
            self.last_key = None;
        }

        match key.code {
            // Vim-style movement
            KeyCode::Char('h') | KeyCode::Left => self.get_current_editor_mut().move_cursor_left(),
            KeyCode::Char('j') | KeyCode::Down => self.get_current_editor_mut().move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.get_current_editor_mut().move_cursor_up(),
            KeyCode::Char('l') | KeyCode::Right => {
                self.get_current_editor_mut().move_cursor_right()
            }

            // Line movement
            KeyCode::Home => self.get_current_editor_mut().move_to_line_start(),
            KeyCode::End => self.get_current_editor_mut().move_to_line_end(),
            KeyCode::PageUp => self.get_current_editor_mut().page_up(),
            KeyCode::PageDown => self.get_current_editor_mut().page_down(),

            // Macro recording and playback
            KeyCode::Char('q') => {
                if self.last_key == Some('q') {
                    // qq - stop recording
                    if self.macro_recording {
                        self.stop_macro_recording();
                    }
                    self.last_key = None;
                } else if self.macro_recording {
                    // Stop recording
                    self.stop_macro_recording();
                } else {
                    // Start recording - wait for register
                    self.last_key = Some('q');
                }
            }
            KeyCode::Char('@') => {
                // Play macro - wait for register
                self.last_key = Some('@');
            }
            KeyCode::Char(c) if self.last_key == Some('q') && !self.macro_recording => {
                // Start recording to register
                self.start_macro_recording(c);
                self.last_key = None;
            }
            KeyCode::Char(c) if self.last_key == Some('@') => {
                // Play macro from register
                self.play_macro(c);
                self.last_key = None;
            }

            // Visual mode
            KeyCode::Char('v') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.get_current_editor_mut().start_visual_selection();
                self.ui_state.enter_visual_mode();
            }
            KeyCode::Char('V') => {
                // Visual line mode (treat as visual for now)
                self.get_current_editor_mut().start_visual_selection();
                self.get_current_editor_mut().move_to_line_start();
                self.ui_state.enter_visual_mode();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Visual block mode
                self.get_current_editor_mut().start_visual_selection();
                self.ui_state.enter_visual_block_mode();
            }

            // Replace mode
            KeyCode::Char('R') => {
                self.ui_state.enter_replace_mode();
            }

            // Insert mode transitions
            KeyCode::Char('i') => self.ui_state.enter_insert_mode(),
            KeyCode::Char('a') => {
                self.get_current_editor_mut().move_cursor_right();
                self.ui_state.enter_insert_mode();
            }
            KeyCode::Char('o') => {
                self.get_current_editor_mut().move_to_line_end();
                self.get_current_editor_mut().insert_newline();
                self.ui_state.enter_insert_mode();
            }
            KeyCode::Char('O') => {
                self.get_current_editor_mut().move_to_line_start();
                self.get_current_editor_mut().insert_newline();
                self.get_current_editor_mut().move_cursor_up();
                self.ui_state.enter_insert_mode();
            }

            // Delete operations
            KeyCode::Char('x') => self.get_current_editor_mut().delete_char_forward(),
            KeyCode::Char('d') => self.handle_delete_command(),

            // Paste
            KeyCode::Char('p') => {
                self.get_current_editor_mut().paste();
                self.ui_state.set_success_message("Text pasted".to_string());
            }

            // Undo/Redo
            KeyCode::Char('u') => self.handle_undo(),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_redo()
            }

            // Mode switches
            KeyCode::Char(':') => self.ui_state.enter_command_mode(),
            KeyCode::Char('/') => self.ui_state.enter_search_mode(),
            KeyCode::Char('?') => self.ui_state.toggle_help(),

            _ => {}
        }
        Ok(())
    }

    fn handle_insert_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.ui_state.enter_normal_mode(),
            KeyCode::Char(c) => self.get_current_editor_mut().insert_char(c),
            KeyCode::Enter => self.get_current_editor_mut().insert_newline(),
            KeyCode::Backspace => self.get_current_editor_mut().delete_char_backward(),
            KeyCode::Delete => self.get_current_editor_mut().delete_char_forward(),
            KeyCode::Tab => self.get_current_editor_mut().insert_tab(),

            // Cursor movement in insert mode
            KeyCode::Left => self.get_current_editor_mut().move_cursor_left(),
            KeyCode::Right => self.get_current_editor_mut().move_cursor_right(),
            KeyCode::Up => self.get_current_editor_mut().move_cursor_up(),
            KeyCode::Down => self.get_current_editor_mut().move_cursor_down(),

            _ => {}
        }
        Ok(())
    }

    fn handle_delete_command(&mut self) {
        // Vim dd command: delete line (second d press)
        if self.last_key == Some('d') {
            self.get_current_editor_mut().delete_line();
            self.ui_state
                .set_success_message("Line deleted and yanked".to_string());
            self.last_key = None;
        } else {
            self.last_key = Some('d');
        }
    }

    fn handle_undo(&mut self) {
        if self.get_current_editor_mut().undo() {
            self.ui_state.set_success_message("Undone".to_string());
        } else {
            self.ui_state
                .set_warning_message("Nothing to undo".to_string());
        }
    }

    fn handle_redo(&mut self) {
        if self.get_current_editor_mut().redo() {
            self.ui_state.set_success_message("Redone".to_string());
        } else {
            self.ui_state
                .set_warning_message("Nothing to redo".to_string());
        }
    }

    fn handle_visual_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit visual mode
            KeyCode::Esc => {
                self.get_current_editor_mut().clear_visual_selection();
                self.ui_state.enter_normal_mode();
            }

            // Movement extends selection
            KeyCode::Char('h') | KeyCode::Left => self.get_current_editor_mut().move_cursor_left(),
            KeyCode::Char('j') | KeyCode::Down => self.get_current_editor_mut().move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.get_current_editor_mut().move_cursor_up(),
            KeyCode::Char('l') | KeyCode::Right => {
                self.get_current_editor_mut().move_cursor_right()
            }
            KeyCode::Home => self.get_current_editor_mut().move_to_line_start(),
            KeyCode::End => self.get_current_editor_mut().move_to_line_end(),

            // Operations on selection
            KeyCode::Char('d') | KeyCode::Char('x') => {
                self.get_current_editor_mut().delete_selection();
                self.ui_state.enter_normal_mode();
                self.ui_state
                    .set_success_message("Selection deleted and yanked".to_string());
            }
            KeyCode::Char('y') => {
                self.get_current_editor_mut().yank_selection();
                self.get_current_editor_mut().clear_visual_selection();
                self.ui_state.enter_normal_mode();
                self.ui_state
                    .set_success_message("Selection yanked".to_string());
            }
            KeyCode::Char('c') => {
                self.get_current_editor_mut().delete_selection();
                self.ui_state.enter_insert_mode();
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_replace_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.ui_state.enter_normal_mode();
            }
            KeyCode::Char(c) => {
                self.get_current_editor_mut().replace_char(c);
            }
            KeyCode::Left => self.get_current_editor_mut().move_cursor_left(),
            KeyCode::Right => self.get_current_editor_mut().move_cursor_right(),
            KeyCode::Up => self.get_current_editor_mut().move_cursor_up(),
            KeyCode::Down => self.get_current_editor_mut().move_cursor_down(),
            _ => {}
        }
        Ok(())
    }

    async fn handle_command_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let command = self.ui_state.get_command_buffer().to_string();

                // Add to history
                self.ui_state.add_to_history(command.clone());

                // Execute command - handle buffer operations in App
                let command_result = {
                    let current_editor = &mut self.buffer_manager.get_current_mut().content;
                    self.command_processor
                        .execute_command(
                            &command,
                            current_editor,
                            &mut self.file_manager,
                            &mut self.config,
                            &mut self.ui_state.should_quit,
                        )
                        .await
                };

                match command_result {
                    Ok(message) => {
                        if !message.is_empty() {
                            // Determine message type based on content
                            if message.contains("saved") || message.contains("Wrote") {
                                self.ui_state.set_success_message(message);
                            } else {
                                self.ui_state.set_info_message(message);
                            }
                        }
                    }
                    Err(e) => {
                        self.ui_state.set_error_message(e.to_string());
                    }
                }

                self.refresh_current_buffer_metadata();
                self.ui_state.enter_normal_mode();
                self.ui_state.clear_command_buffer();
            }
            KeyCode::Esc => {
                self.ui_state.clear_command_buffer();
                self.ui_state.enter_normal_mode();
                self.ui_state.set_info_message("Cancelled".to_string());
            }
            KeyCode::Up => {
                // Navigate command history up
                self.ui_state.history_up();
            }
            KeyCode::Down => {
                // Navigate command history down
                self.ui_state.history_down();
            }
            KeyCode::Tab => {
                // Command completion
                let current = self.ui_state.get_command_buffer();
                let suggestions = self.ui_state.get_command_suggestions(current);
                if suggestions.len() == 1 {
                    self.ui_state.set_command_buffer(suggestions[0].clone());
                } else if suggestions.len() > 1 {
                    // Show suggestions in status
                    let msg = format!("Suggestions: {}", suggestions.join(", "));
                    self.ui_state.set_info_message(msg);
                }
            }
            KeyCode::Char(c) => {
                self.ui_state.push_to_command_buffer(c);
            }
            KeyCode::Backspace => {
                self.ui_state.pop_from_command_buffer();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.ui_state.hide_help();
            }
            KeyCode::Esc => {
                self.ui_state.hide_help();
            }
            _ => {}
        }
        Ok(())
    }

    // Helper methods for main.rs

    pub fn set_mode(&mut self, mode: Mode) {
        self.ui_state.set_mode(mode);
    }

    pub fn update_status(&mut self) {
        self.ui_state.update_status();
    }

    // Handle save prompt responses
    async fn handle_save_prompt_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let save_result = {
                    let current_editor = &mut self.buffer_manager.get_current_mut().content;
                    self.file_manager.save_file(current_editor).await
                };

                if let Err(e) = save_result {
                    self.ui_state
                        .set_error_message(format!("Error saving: {}", e));
                    self.ui_state.enter_normal_mode();
                } else {
                    self.refresh_current_buffer_metadata();
                    self.ui_state.quit();
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.ui_state.quit();
            }
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
                self.ui_state.enter_normal_mode();
                self.ui_state.status_message.clear();
            }
            _ => {}
        }
        Ok(())
    }

    // Macro recording methods
    fn start_macro_recording(&mut self, register: char) {
        self.macro_recording = true;
        self.macro_register = Some(register);
        self.macro_keys.clear();
        self.ui_state
            .set_info_message(format!("Recording macro to register '{}'", register));
    }

    fn stop_macro_recording(&mut self) {
        if let Some(register) = self.macro_register {
            self.macro_registers
                .insert(register, self.macro_keys.clone());
            self.ui_state
                .set_success_message(format!("Macro recorded to register '{}'", register));
        }
        self.macro_recording = false;
        self.macro_register = None;
        self.macro_keys.clear();
    }

    fn play_macro(&mut self, register: char) {
        if let Some(keys) = self.macro_registers.get(&register).cloned() {
            self.ui_state
                .set_info_message(format!("Playing macro from register '{}'", register));
            // Execute recorded keys
            for key in keys {
                // Skip the macro recording keys themselves
                if key.code != KeyCode::Char('q') {
                    // Process the key based on current mode
                    let _ = match self.ui_state.get_mode() {
                        Mode::Normal => self.handle_normal_mode_key(key),
                        Mode::Insert => self.handle_insert_mode_key(key),
                        Mode::Visual | Mode::VisualBlock => self.handle_visual_mode_key(key),
                        Mode::Replace => self.handle_replace_mode_key(key),
                        _ => Ok(()),
                    };
                }
            }
            self.ui_state
                .set_success_message("Macro playback complete".to_string());
        } else {
            self.ui_state
                .set_warning_message(format!("No macro in register '{}'", register));
        }
    }

    fn refresh_current_buffer_metadata(&mut self) {
        if let Some(buffer) = self
            .buffer_manager
            .buffers
            .get_mut(self.buffer_manager.current_buffer)
        {
            buffer.file_path = self.file_manager.get_current_path().cloned();
            buffer.readonly = self.file_manager.is_readonly();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[tokio::test]
    async fn test_app_creation() {
        let app = App::new().await;
        assert!(app.is_ok());

        let app = app.unwrap();
        assert!(matches!(app.mode(), &Mode::Normal));
        assert!(!app.is_modified());
        assert!(!app.should_quit());
        assert_eq!(app.file_path(), None);
    }

    #[tokio::test]
    async fn test_mode_switching() {
        let mut app = App::new().await.unwrap();

        app.set_mode(Mode::Insert);
        assert!(matches!(app.mode(), &Mode::Insert));

        app.set_mode(Mode::Command);
        assert!(matches!(app.mode(), &Mode::Command));
    }

    #[tokio::test]
    async fn test_status_message_setting() {
        let mut app = App::new().await.unwrap();

        app.ui_state.set_info_message("Test message".to_string());
        assert_eq!(app.status_message(), "Test message");
    }

    #[tokio::test]
    async fn test_command_execution_quit() {
        let mut app = App::new().await.unwrap();
        app.ui_state.set_mode(Mode::Command);
        app.ui_state.set_command_buffer("q".to_string());

        // Test quit directly
        app.quit();
        assert!(app.should_quit()); // Should quit since no modifications
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let mut app = App::new().await.unwrap();
        app.get_current_editor_mut()
            .set_content("Hello World\nTest line".to_string());

        // Test search directly on editor
        app.get_current_editor_mut().search("World");

        // Check cursor moved to found position
        let (line, col) = app.get_current_editor().cursor_position();
        assert_eq!(line, 0);
        assert_eq!(col, 6); // "World" starts at column 6 in "Hello World"
    }

    #[tokio::test]
    async fn test_insert_mode_key_handling() {
        let mut app = App::new().await.unwrap();
        app.ui_state.set_mode(Mode::Insert);

        // Test character insertion
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('H')))
            .await;
        assert!(result.is_ok());
        assert_eq!(app.get_current_editor().get_content(), "H");

        // Test escape to normal mode
        let result = app.handle_editor_key(create_key_event(KeyCode::Esc)).await;
        assert!(result.is_ok());
        assert!(matches!(app.mode(), &Mode::Normal));
    }

    #[tokio::test]
    async fn test_normal_mode_vim_commands() {
        let mut app = App::new().await.unwrap();
        app.get_current_editor_mut()
            .set_content("Hello World".to_string());

        // Test 'i' to enter insert mode
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('i')))
            .await;
        assert!(result.is_ok());
        assert!(matches!(app.mode(), &Mode::Insert));
        assert_eq!(app.status_message(), "-- INSERT --");

        // Reset to normal mode
        app.ui_state.enter_normal_mode();

        // Test ':' to enter command mode
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char(':')))
            .await;
        assert!(result.is_ok());
        assert!(matches!(app.mode(), &Mode::Command));
        assert_eq!(app.status_message(), ":");
    }

    #[tokio::test]
    async fn test_file_operations() {
        let mut app = App::new().await.unwrap();

        // Test that initially no file is loaded
        assert!(app.file_path().is_none());

        // Test quit functionality
        app.quit();
        assert!(app.should_quit());
    }

    #[tokio::test]
    async fn test_vim_operations() {
        let mut app = App::new().await.unwrap();

        // Test undo operation
        app.get_current_editor_mut().insert_char('a');
        app.get_current_editor_mut().insert_char('b');
        assert_eq!(app.get_current_editor().get_content(), "ab");

        // Test undo through normal mode key
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('u')))
            .await;
        assert!(result.is_ok());
        assert_eq!(app.get_current_editor().get_content(), "a");

        // Test redo
        let redo_key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let result = app.handle_editor_key(redo_key).await;
        assert!(result.is_ok());
        assert_eq!(app.get_current_editor().get_content(), "ab");
    }

    #[tokio::test]
    async fn test_command_history() {
        let mut app = App::new().await.unwrap();

        // Enter command mode
        app.ui_state.enter_command_mode();

        // Add some commands to history
        app.ui_state.set_command_buffer("w".to_string());
        app.ui_state.add_to_history("w".to_string());

        app.ui_state.set_command_buffer("q".to_string());
        app.ui_state.add_to_history("q".to_string());

        app.ui_state.set_command_buffer("wq".to_string());
        app.ui_state.add_to_history("wq".to_string());

        // Clear buffer and navigate history
        app.ui_state.clear_command_buffer();

        // Go up in history
        app.ui_state.history_up();
        assert_eq!(app.ui_state.get_command_buffer(), "wq");

        app.ui_state.history_up();
        assert_eq!(app.ui_state.get_command_buffer(), "q");

        app.ui_state.history_up();
        assert_eq!(app.ui_state.get_command_buffer(), "w");

        // Go down in history
        app.ui_state.history_down();
        assert_eq!(app.ui_state.get_command_buffer(), "q");
    }

    #[tokio::test]
    async fn test_command_completion() {
        let app = App::new().await.unwrap();

        // Test command suggestions
        let suggestions = app.ui_state.get_command_suggestions("w");
        assert!(suggestions.contains(&"w".to_string()));
        assert!(suggestions.contains(&"wq".to_string()));

        let suggestions = app.ui_state.get_command_suggestions("q");
        assert!(suggestions.contains(&"q".to_string()));
        assert!(suggestions.contains(&"q!".to_string()));

        let suggestions = app.ui_state.get_command_suggestions("e");
        assert!(suggestions.contains(&"e".to_string()));
    }

    #[tokio::test]
    async fn test_macro_recording() {
        let mut app = App::new().await.unwrap();

        // Start recording to register 'a'
        app.start_macro_recording('a');
        assert!(app.macro_recording);
        assert_eq!(app.macro_register, Some('a'));

        // Record some keys
        app.macro_keys.push(create_key_event(KeyCode::Char('i')));
        app.macro_keys.push(create_key_event(KeyCode::Char('h')));
        app.macro_keys.push(create_key_event(KeyCode::Char('i')));
        app.macro_keys.push(create_key_event(KeyCode::Esc));

        // Stop recording
        app.stop_macro_recording();
        assert!(!app.macro_recording);
        assert_eq!(app.macro_register, None);

        // Check macro was saved
        assert!(app.macro_registers.contains_key(&'a'));
        assert_eq!(app.macro_registers.get(&'a').unwrap().len(), 4);
    }

    #[tokio::test]
    async fn test_macro_playback() {
        let mut app = App::new().await.unwrap();

        // Record a simple macro
        app.start_macro_recording('b');
        app.macro_keys.push(create_key_event(KeyCode::Char('x')));
        app.stop_macro_recording();

        // Set up editor with content
        app.get_current_editor_mut()
            .set_content("Hello World".to_string());

        // Play macro
        app.play_macro('b');

        // Check that 'x' command was executed (delete char)
        assert_eq!(app.get_current_editor().get_content(), "ello World");

        // Try playing non-existent macro
        app.play_macro('z');
        assert!(app.ui_state.get_status_message().contains("No macro"));
    }

    #[tokio::test]
    async fn test_dd_command() {
        let mut app = App::new().await.unwrap();
        app.get_current_editor_mut().set_content(
            "Line 1
Line 2
Line 3"
                .to_string(),
        );

        // First 'd' should not delete
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('d')))
            .await;
        assert!(result.is_ok());
        assert_eq!(
            app.get_current_editor().get_content(),
            "Line 1
Line 2
Line 3"
        );
        assert_eq!(app.last_key, Some('d'));

        // Second 'd' should delete the line
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('d')))
            .await;
        assert!(result.is_ok());
        assert_eq!(
            app.get_current_editor().get_content(),
            "Line 2
Line 3"
        );
        assert_eq!(app.last_key, None);

        // Test paste after dd (paste puts the line at cursor position)
        let result = app
            .handle_editor_key(create_key_event(KeyCode::Char('p')))
            .await;
        assert!(result.is_ok());
        // The pasted line should be inserted at current position
        let content = app.get_current_editor().get_content();
        assert!(content.contains("Line 1")); // Original line 1 should still be in content
    }
}
