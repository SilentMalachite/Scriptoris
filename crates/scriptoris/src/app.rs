//! アプリケーション全体の状態とキー操作を扱うモジュール。
//! バッファやウィンドウ、マクロ、UI 状態などエディタの中枢がここに集約されます。

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};

use crate::command_processor::{BufferCommand, CommandAction, CommandProcessor, WindowCommand};
use crate::config::Config;
use crate::editor::Editor;
use crate::file_manager::FileManager;
use crate::highlight::Highlighter;
use crate::text_width::{EmojiWidth, TextWidthCalculator};
use crate::ui_state::UIState;

// LSP integration
#[cfg(feature = "lsp")]
use lsp_plugin::LspPlugin;

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
    highlighter_cache: Option<Highlighter>, // Cache highlighter
    last_key: Option<char>, // For handling multi-key commands like dd
    // Macro recording
    macro_recording: bool,
    macro_register: Option<char>,
    macro_keys: Vec<KeyEvent>,
    macro_registers: std::collections::HashMap<char, Vec<KeyEvent>>,
    // Cross-platform text width calculator for accurate cursor positioning
    pub text_calculator: TextWidthCalculator,
    // LSP integration for enhanced syntax highlighting and code intelligence
    #[cfg(feature = "lsp")]
    lsp_plugin: Option<LspPlugin>,
}

// バッファ管理
pub struct Buffer {
    pub id: usize,
    pub content: Editor,
    pub file_path: Option<PathBuf>,
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
    next_buffer_id: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        let mut manager = Self {
            buffers: Vec::new(),
            current_buffer: 0,
            next_buffer_id: 0,
        };
        manager.create_buffer();
        manager
    }

    fn create_buffer(&mut self) -> usize {
        let id = self.next_buffer_id;
        self.next_buffer_id += 1;
        self.buffers.push(Buffer::new(id));
        self.buffers.len() - 1
    }

    pub fn get_current(&self) -> &Buffer {
        &self.buffers[self.current_buffer]
    }

    pub fn get_current_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current_buffer]
    }

    pub fn current_buffer_id(&self) -> usize {
        self.get_current().id
    }

    pub fn buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    pub fn find_index_by_id(&self, id: usize) -> Option<usize> {
        self.buffers.iter().position(|buffer| buffer.id == id)
    }

    pub fn current_index(&self) -> usize {
        self.current_buffer
    }

    pub fn next_buffer(&mut self) -> Option<&mut Buffer> {
        if self.buffers.len() <= 1 {
            return None;
        }
        self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
        Some(&mut self.buffers[self.current_buffer])
    }

    pub fn prev_buffer(&mut self) -> Option<&mut Buffer> {
        if self.buffers.len() <= 1 {
            return None;
        }
        if self.current_buffer == 0 {
            self.current_buffer = self.buffers.len() - 1;
        } else {
            self.current_buffer -= 1;
        }
        Some(&mut self.buffers[self.current_buffer])
    }

    pub fn delete_current(&mut self) -> Option<usize> {
        if self.buffers.len() <= 1 {
            let buffer = &mut self.buffers[self.current_buffer];
            buffer.content = Editor::default();
            buffer.file_path = None;
            buffer.readonly = false;
            None
        } else {
            let removed = self.buffers.remove(self.current_buffer);
            if self.current_buffer >= self.buffers.len() {
                self.current_buffer = self.buffers.len().saturating_sub(1);
            }
            Some(removed.id)
        }
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new()
    }
}

// ウィンドウ管理
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSplitKind {
    None,
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct WindowPane {
    pub id: usize,
    pub buffer_id: usize,
}

pub struct WindowManager {
    panes: Vec<WindowPane>,
    pub current_window_id: usize,
    next_window_id: usize,
    split: WindowSplitKind,
}

// プラグインシステム
#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, app: &mut App) -> Result<()>;
    fn on_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool>;
    fn on_command(&mut self, app: &mut App, command: &str) -> Result<Option<String>>;
    fn on_save(&mut self, app: &mut App, path: &Path) -> Result<()>;
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

    pub fn handle_save(&mut self, app: &mut App, path: &Path) -> Result<()> {
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

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            panes: vec![WindowPane { id: 0, buffer_id }],
            current_window_id: 0,
            next_window_id: 1,
            split: WindowSplitKind::None,
        }
    }

    pub fn panes(&self) -> &[WindowPane] {
        &self.panes
    }

    pub fn panes_mut(&mut self) -> &mut [WindowPane] {
        &mut self.panes
    }

    fn split(&mut self, buffer_id: usize, kind: WindowSplitKind) {
        if self.panes.len() < 2 {
            self.panes.push(WindowPane {
                id: self.next_window_id,
                buffer_id,
            });
            self.next_window_id += 1;
        } else if let Some(pane) = self
            .panes
            .iter_mut()
            .find(|pane| pane.id != self.current_window_id)
        {
            pane.buffer_id = buffer_id;
        }
        self.split = kind;
    }

    pub fn split_horizontal(&mut self, buffer_id: usize) {
        self.split(buffer_id, WindowSplitKind::Horizontal);
    }

    pub fn split_vertical(&mut self, buffer_id: usize) {
        self.split(buffer_id, WindowSplitKind::Vertical);
    }

    pub fn set_buffer_for_current(&mut self, buffer_id: usize) {
        if let Some(pane) = self
            .panes
            .iter_mut()
            .find(|pane| pane.id == self.current_window_id)
        {
            pane.buffer_id = buffer_id;
        }
    }

    pub fn split_kind(&self) -> WindowSplitKind {
        self.split
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiMessageKind {
    Info,
    Success,
    Warning,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::load().await?;
        let mut buffer_manager = BufferManager::new();

        // Apply tab configuration to the initial buffer
        buffer_manager
            .get_current_mut()
            .content
            .set_tab_config(config.editor.tab_size, config.editor.use_spaces);

        let initial_buffer_id = buffer_manager.get_current().id;
        let command_processor = CommandProcessor::new()?;

        // Initialize LSP plugin if feature is enabled
        #[cfg(feature = "lsp")]
        let lsp_plugin = {
            let plugin = LspPlugin::new();
            log::info!("LSP plugin initialized successfully");
            Some(plugin)
        };

        Ok(Self {
            config,
            ui_state: UIState::new(),
            file_manager: FileManager::new(),
            command_processor,
            buffer_manager,
            window_manager: WindowManager::new(initial_buffer_id),
            highlighter_cache: None,
            last_key: None,
            macro_recording: false,
            macro_register: None,
            macro_keys: Vec::new(),
            macro_registers: std::collections::HashMap::new(),
            // Initialize text calculator for cross-platform compatibility
            text_calculator: TextWidthCalculator::new()
                .east_asian_aware(true)
                .emoji_width(EmojiWidth::Standard),
            #[cfg(feature = "lsp")]
            lsp_plugin,
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

    pub fn get_highlighter(&mut self) -> &Highlighter {
        let needs_refresh = self
            .highlighter_cache
            .as_ref()
            .map(|cache| cache.theme_name() != self.config.theme.syntax_theme.as_str())
            .unwrap_or(true);

        if needs_refresh {
            self.highlighter_cache = Some(Highlighter::new(
                &self.config.theme.syntax_theme,
            ));
        }

        self.highlighter_cache
            .as_ref()
            .expect("Highlighter cache should be initialised")
    }

    // Public getters for UI and main.rs
    pub fn should_quit(&self) -> bool {
        self.ui_state.should_quit()
    }

    // Legacy properties for backward compatibility
    pub fn show_help(&self) -> bool {
        self.ui_state.is_help_shown()
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
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
            self.macro_keys.push(key);
        }

        // Clear last_key if it's not a command key and we're not pressing that key
        if key.code != KeyCode::Char('d')
            && key.code != KeyCode::Char('y')
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

            // Yank operations
            KeyCode::Char('y') => self.handle_yank_command(),

            // Paste
            KeyCode::Char('p') => {
                self.get_current_editor_mut().paste();
                self.ui_state
                    .set_success_message("貼り付けました".to_string());
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
                .set_success_message("行を削除してヤンクしました".to_string());
            self.last_key = None;
        } else {
            self.last_key = Some('d');
        }
    }

    fn handle_yank_command(&mut self) {
        // Vim yy command: yank line (second y press)
        if self.last_key == Some('y') {
            self.get_current_editor_mut().yank_line();
            self.ui_state
                .set_success_message("行をヤンクしました".to_string());
            self.last_key = None;
        } else {
            self.last_key = Some('y');
        }
    }

    fn handle_undo(&mut self) {
        if self.get_current_editor_mut().undo() {
            self.ui_state
                .set_success_message("元に戻しました".to_string());
        } else {
            self.ui_state
                .set_warning_message("元に戻す操作はありません".to_string());
        }
    }

    fn handle_redo(&mut self) {
        if self.get_current_editor_mut().redo() {
            self.ui_state
                .set_success_message("やり直しました".to_string());
        } else {
            self.ui_state
                .set_warning_message("やり直す操作はありません".to_string());
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
                    .set_success_message("選択範囲を削除してヤンクしました".to_string());
            }
            KeyCode::Char('y') => {
                self.get_current_editor_mut().yank_selection();
                self.get_current_editor_mut().clear_visual_selection();
                self.ui_state.enter_normal_mode();
                self.ui_state
                    .set_success_message("選択範囲をヤンクしました".to_string());
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

                let mut message_to_show: Option<(UiMessageKind, String)> = match command_result {
                    Ok(message) if !message.is_empty() => {
                        Some((classify_message(&message), message))
                    }
                    Ok(_) => None,
                    Err(e) => {
                        self.ui_state.set_error_message(e.to_string());
                        None
                    }
                };

                if let Some(action) = self.command_processor.take_pending_action() {
                    if let Some(action_message) = self.apply_command_action(action) {
                        message_to_show = Some(action_message);
                    }
                }

                if let Some((kind, message)) = message_to_show {
                    match kind {
                        UiMessageKind::Info => self.ui_state.set_info_message(message),
                        UiMessageKind::Success => self.ui_state.set_success_message(message),
                        UiMessageKind::Warning => self.ui_state.set_warning_message(message),
                    }
                }

                self.refresh_current_buffer_metadata();
                self.ui_state.enter_normal_mode();
                self.ui_state.clear_command_buffer();
            }
            KeyCode::Esc => {
                self.ui_state.clear_command_buffer();
                self.ui_state.enter_normal_mode();
                self.ui_state
                    .set_info_message("キャンセルしました".to_string());
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
                match suggestions.len() {
                    0 => {}
                    1 => self.ui_state.set_command_buffer(suggestions[0].clone()),
                    _ => {
                        let msg = format!("候補: {}", suggestions.join(", "));
                        self.ui_state.set_info_message(msg);
                    }
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
                        .set_error_message(format!("保存中にエラーが発生しました: {}", e));
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
            .set_info_message(format!("レジスタ '{}' にマクロを記録中", register));
    }

    fn stop_macro_recording(&mut self) {
        if let Some(register) = self.macro_register {
            self.macro_registers
                .insert(register, self.macro_keys.clone());
            self.ui_state
                .set_success_message(format!("レジスタ '{}' にマクロを記録しました", register));
        }
        self.macro_recording = false;
        self.macro_register = None;
        self.macro_keys.clear();
    }

    fn play_macro(&mut self, register: char) {
        if let Some(keys) = self.macro_registers.get(&register).cloned() {
            self.ui_state
                .set_info_message(format!("レジスタ '{}' のマクロを再生します", register));
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
                .set_success_message("マクロの再生が完了しました".to_string());
        } else {
            self.ui_state.set_warning_message(format!(
                "レジスタ '{}' にマクロは登録されていません",
                register
            ));
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

    fn sync_file_manager_from_buffer(&mut self) {
        let buffer = self.buffer_manager.get_current();
        self.file_manager.current_path = buffer.file_path.clone();
        self.file_manager.is_readonly = buffer.readonly;
    }

    // LSP integration methods
    #[cfg(feature = "lsp")]
    pub async fn notify_lsp_document_opened(&self, path: &Path, content: &str) {
        if let Some(ref lsp_plugin) = self.lsp_plugin {
            let language_id = self.detect_language_id(path);
            if let Err(e) = lsp_plugin
                .open_document(path.to_path_buf(), content.to_string(), language_id)
                .await
            {
                log::warn!("Failed to notify LSP of document open: {}", e);
            }
        }
    }

    #[cfg(feature = "lsp")]
    #[allow(dead_code)]
    pub async fn notify_lsp_document_changed(
        &self,
        path: &Path,
        content: &str,
        version: i32,
    ) {
        if let Some(ref lsp_plugin) = self.lsp_plugin {
            if let Err(e) = lsp_plugin
                .update_document(path.to_path_buf(), content.to_string(), version)
                .await
            {
                log::warn!("Failed to notify LSP of document change: {}", e);
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_lsp_completions(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Vec<lsp_types::CompletionItem> {
        #[cfg(feature = "lsp")]
        if let Some(ref lsp_plugin) = self.lsp_plugin {
            return lsp_plugin
                .get_completions(path.to_path_buf(), line, character)
                .await
                .unwrap_or_default();
        }

        #[cfg(not(feature = "lsp"))]
        {
            let _ = (path, line, character);
        }

        vec![]
    }

    fn detect_language_id(&self, path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("md") => "markdown".to_string(),
            Some("js") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("jsx") => "javascriptreact".to_string(),
            Some("tsx") => "typescriptreact".to_string(),
            Some("py") => "python".to_string(),
            Some("json") => "json".to_string(),
            Some("yaml") | Some("yml") => "yaml".to_string(),
            Some("toml") => "toml".to_string(),
            Some("html") => "html".to_string(),
            Some("css") => "css".to_string(),
            Some("sql") => "sql".to_string(),
            Some("sh") => "shellscript".to_string(),
            Some("txt") => "plaintext".to_string(),
            _ => "plaintext".to_string(),
        }
    }

    fn handle_buffer_switch(&mut self, message: &str) -> Option<(UiMessageKind, String)> {
        let active_id = self.buffer_manager.current_buffer_id();
        self.window_manager.set_buffer_for_current(active_id);
        self.sync_file_manager_from_buffer();
        Some((UiMessageKind::Info, message.to_string()))
    }

    fn apply_command_action(&mut self, action: CommandAction) -> Option<(UiMessageKind, String)> {
        match action {
            CommandAction::None => None,
            CommandAction::Buffer(buffer_command) => match buffer_command {
                BufferCommand::Next => {
                    if self.buffer_manager.next_buffer().is_some() {
                        self.handle_buffer_switch("次のバッファに切り替えました")
                    } else {
                        Some((
                            UiMessageKind::Warning,
                            "切り替え可能なバッファがありません".to_string(),
                        ))
                    }
                }
                BufferCommand::Previous => {
                    if self.buffer_manager.prev_buffer().is_some() {
                        self.handle_buffer_switch("前のバッファに切り替えました")
                    } else {
                        Some((
                            UiMessageKind::Warning,
                            "切り替え可能なバッファがありません".to_string(),
                        ))
                    }
                }
                BufferCommand::List => {
                    let list = self
                        .buffer_manager
                        .buffers()
                        .iter()
                        .enumerate()
                        .map(|(index, buffer)| {
                            let marker = if index == self.buffer_manager.current_index() {
                                "*"
                            } else {
                                " "
                            };
                            let name = buffer
                                .file_path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("[No Name]");
                            format!("{}{}:{name}", marker, buffer.id)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    Some((UiMessageKind::Info, format!("バッファ一覧: {list}")))
                }
                BufferCommand::DeleteCurrent => {
                    let removed_id = self.buffer_manager.delete_current();
                    let active_id = self.buffer_manager.current_buffer_id();
                    self.window_manager.set_buffer_for_current(active_id);
                    if let Some(old_id) = removed_id {
                        for pane in self.window_manager.panes_mut() {
                            if pane.buffer_id == old_id {
                                pane.buffer_id = active_id;
                            }
                        }
                    }
                    self.sync_file_manager_from_buffer();
                    if removed_id.is_some() {
                        Some((
                            UiMessageKind::Success,
                            "現在のバッファを閉じました".to_string(),
                        ))
                    } else {
                        Some((
                            UiMessageKind::Warning,
                            "最後のバッファを初期化しました".to_string(),
                        ))
                    }
                }
            },
            CommandAction::Window(window_command) => match window_command {
                WindowCommand::SplitHorizontal => {
                    let buffer_id = self.buffer_manager.current_buffer_id();
                    self.window_manager.split_horizontal(buffer_id);
                    Some((UiMessageKind::Info, "水平分割を行いました".to_string()))
                }
                WindowCommand::SplitVertical => {
                    let buffer_id = self.buffer_manager.current_buffer_id();
                    self.window_manager.split_vertical(buffer_id);
                    Some((UiMessageKind::Info, "垂直分割を行いました".to_string()))
                }
            },
        }
    }
}

fn classify_message(message: &str) -> UiMessageKind {
    if message.contains("書き込みました") || message.contains("保存しました") {
        UiMessageKind::Success
    } else if message.contains("変更が保存されていません") {
        UiMessageKind::Warning
    } else {
        UiMessageKind::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_manager::SessionManager;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;
    use tokio::fs::try_exists;

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn restore_env(name: &str, previous: Option<String>) {
        if let Some(value) = previous {
            std::env::set_var(name, value);
        } else {
            std::env::remove_var(name);
        }
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
    async fn test_app_creation_with_env_overrides() {
        let (prev_config_dir, prev_config_path, prev_data_dir, config_dir, _data_dir) = {
            let _guard = env_lock().lock().unwrap();
            let prev_config_dir = std::env::var("SCRIPTORIS_CONFIG_DIR").ok();
            let prev_config_path = std::env::var("SCRIPTORIS_CONFIG_PATH").ok();
            let prev_data_dir = std::env::var("SCRIPTORIS_DATA_DIR").ok();

            let config_dir = TempDir::new().unwrap();
            let data_dir = TempDir::new().unwrap();

            std::env::set_var("SCRIPTORIS_CONFIG_DIR", config_dir.path());
            std::env::remove_var("SCRIPTORIS_CONFIG_PATH");
            std::env::set_var("SCRIPTORIS_DATA_DIR", data_dir.path());
            (
                prev_config_dir,
                prev_config_path,
                prev_data_dir,
                config_dir,
                data_dir,
            )
        }; // release lock before await

        let app = App::new()
            .await
            .expect("app should initialize with environment overrides");

        let config_path = config_dir.path().join("config.json");
        assert!(try_exists(&config_path)
            .await
            .expect("config path existence check should succeed"));

        drop(app);

        let session_manager =
            SessionManager::new().expect("session manager should use overridden data directory");

        let editor = Editor::new();
        let file_manager = FileManager::new();
        let config = Config::default();

        let result = session_manager
            .save_session("envtest", &editor, &file_manager, &config)
            .await;
        assert!(result.is_ok());

        let sessions = session_manager
            .list_sessions()
            .await
            .expect("session list should load");
        assert!(sessions.iter().any(|session| session.name == "envtest"));

        restore_env("SCRIPTORIS_CONFIG_DIR", prev_config_dir);
        restore_env("SCRIPTORIS_CONFIG_PATH", prev_config_path);
        restore_env("SCRIPTORIS_DATA_DIR", prev_data_dir);
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
        assert!(app
            .ui_state
            .get_status_message()
            .contains("マクロは登録されていません"));
    }

    #[tokio::test]
    async fn test_macro_playback_empty_register() {
        let mut app = App::new().await.unwrap();

        app.start_macro_recording('c');
        // すぐに停止して空のマクロを作成
        app.stop_macro_recording();

        app.play_macro('c');
        assert!(app
            .ui_state
            .get_status_message()
            .contains("マクロの再生が完了しました"));
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
