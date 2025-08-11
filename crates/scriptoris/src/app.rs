use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Serialize, Deserialize};



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
    Visual,       // Visual selection mode
    VisualBlock,  // Visual block (rectangular) selection mode  
    Replace,      // Replace mode
    Help,
    SavePrompt,
}

pub struct App {
    pub editor: Editor,
    pub config: Config,
    pub ui_state: UIState,
    pub file_manager: FileManager,
    pub command_processor: CommandProcessor,
    pub buffer_manager: BufferManager,
    pub window_manager: WindowManager,
    pub session_manager: SessionManager,
    pub plugin_manager: PluginManager,
    last_key: Option<char>,  // For handling multi-key commands like dd
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
    pub modified: bool,
    pub readonly: bool,
}

impl Buffer {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            content: Editor::default(),
            file_path: None,
            modified: false,
            readonly: false,
        }
    }

    pub fn from_file(id: usize, path: std::path::PathBuf, content: String) -> Result<Self> {
        let mut editor = Editor::default();
        editor.set_content(content);
        
        let metadata = std::fs::metadata(&path)?;
        let readonly = metadata.permissions().readonly();
        
        Ok(Self {
            id,
            content: editor,
            file_path: Some(path),
            modified: false,
            readonly,
        })
    }
}

pub struct BufferManager {
    buffers: Vec<Buffer>,
    current_buffer: usize,
    next_id: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        let mut manager = Self {
            buffers: Vec::new(),
            current_buffer: 0,
            next_id: 1,
        };
        // デフォルトバッファを作成
        manager.buffers.push(Buffer::new(0));
        manager
    }

    pub fn add_buffer(&mut self, buffer: Buffer) -> usize {
        let id = buffer.id;
        self.buffers.push(buffer);
        self.next_id += 1;
        id
    }

    pub fn create_buffer(&mut self) -> usize {
        let buffer = Buffer::new(self.next_id);
        self.add_buffer(buffer)
    }

    pub fn open_file(&mut self, path: std::path::PathBuf) -> Result<usize> {
        // 既存バッファチェック
        for (i, buffer) in self.buffers.iter().enumerate() {
            if let Some(ref buf_path) = buffer.file_path {
                if buf_path == &path {
                    self.current_buffer = i;
                    return Ok(buffer.id);
                }
            }
        }

        // 新規バッファ作成
        let content = std::fs::read_to_string(&path)?;
        let buffer = Buffer::from_file(self.next_id, path, content)?;
        let id = self.add_buffer(buffer);
        self.current_buffer = self.buffers.len() - 1;
        Ok(id)
    }

    pub fn close_buffer(&mut self, id: usize) -> Result<()> {
        let index = self.buffers.iter().position(|b| b.id == id)
            .ok_or_else(|| anyhow::anyhow!("Buffer not found"))?;
        
        // 最後のバッファは閉じない
        if self.buffers.len() == 1 {
            return Err(anyhow::anyhow!("Cannot close last buffer"));
        }

        self.buffers.remove(index);
        
        // カレントバッファを調整
        if self.current_buffer >= self.buffers.len() {
            self.current_buffer = self.buffers.len() - 1;
        }
        
        Ok(())
    }

    pub fn get_current(&self) -> &Buffer {
        &self.buffers[self.current_buffer]
    }

    pub fn get_current_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current_buffer]
    }

    pub fn switch_to_buffer(&mut self, id: usize) -> Result<()> {
        let index = self.buffers.iter().position(|b| b.id == id)
            .ok_or_else(|| anyhow::anyhow!("Buffer not found"))?;
        self.current_buffer = index;
        Ok(())
    }

    pub fn next_buffer(&mut self) {
        self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
    }

    pub fn prev_buffer(&mut self) {
        if self.current_buffer == 0 {
            self.current_buffer = self.buffers.len() - 1;
        } else {
            self.current_buffer -= 1;
        }
    }

    pub fn list_buffers(&self) -> Vec<(usize, Option<&std::path::Path>, bool, bool)> {
        self.buffers.iter().map(|b| {
            (b.id, b.file_path.as_deref(), b.modified, b.id == self.buffers[self.current_buffer].id)
        }).collect()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.buffers.iter().any(|b| b.modified)
    }

    pub fn get_unsaved_buffers(&self) -> Vec<&Buffer> {
        self.buffers.iter().filter(|b| b.modified).collect()
    }
}

// ウィンドウ管理
#[derive(Clone, Debug)]
pub enum Split {
    Horizontal { top: Box<Window>, bottom: Box<Window>, ratio: f32 },
    Vertical { left: Box<Window>, right: Box<Window>, ratio: f32 },
    Leaf { buffer_id: usize },
}

#[derive(Clone, Debug)]
pub struct Window {
    pub id: usize,
    pub split: Split,
    pub viewport_offset: usize,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

impl Window {
    pub fn new_leaf(id: usize, buffer_id: usize) -> Self {
        Self {
            id,
            split: Split::Leaf { buffer_id },
            viewport_offset: 0,
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn split_horizontal(&mut self, new_id: usize, buffer_id: usize) {
        let old_split = std::mem::replace(&mut self.split, Split::Leaf { buffer_id: 0 });
        self.split = Split::Horizontal {
            top: Box::new(Window {
                id: self.id,
                split: old_split,
                viewport_offset: self.viewport_offset,
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            }),
            bottom: Box::new(Window::new_leaf(new_id, buffer_id)),
            ratio: 0.5,
        };
    }

    pub fn split_vertical(&mut self, new_id: usize, buffer_id: usize) {
        let old_split = std::mem::replace(&mut self.split, Split::Leaf { buffer_id: 0 });
        self.split = Split::Vertical {
            left: Box::new(Window {
                id: self.id,
                split: old_split,
                viewport_offset: self.viewport_offset,
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            }),
            right: Box::new(Window::new_leaf(new_id, buffer_id)),
            ratio: 0.5,
        };
    }
}

pub struct WindowManager {
    root: Window,
    pub current_window_id: usize,
    next_window_id: usize,
}

// セッション管理

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionData {
    pub buffers: Vec<SessionBuffer>,
    pub windows: SessionWindow,
    pub current_buffer: usize,
    pub current_window: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionBuffer {
    pub id: usize,
    pub file_path: Option<std::path::PathBuf>,
    pub content: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub viewport_offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SessionWindow {
    Leaf { buffer_id: usize },
    HSplit { top: Box<SessionWindow>, bottom: Box<SessionWindow>, ratio: f32 },
    VSplit { left: Box<SessionWindow>, right: Box<SessionWindow>, ratio: f32 },
}

pub struct SessionManager {
    session_dir: std::path::PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let session_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("scriptoris")
            .join("sessions");
        
        std::fs::create_dir_all(&session_dir)?;
        
        Ok(Self { session_dir })
    }

    pub fn save_session(&self, name: &str, app: &App) -> Result<()> {
        let session_data = self.create_session_data(app)?;
        let session_path = self.session_dir.join(format!("{}.json", name));
        
        let json = serde_json::to_string_pretty(&session_data)?;
        std::fs::write(session_path, json)?;
        
        Ok(())
    }

    pub fn load_session(&self, name: &str, app: &mut App) -> Result<()> {
        let session_path = self.session_dir.join(format!("{}.json", name));
        let json = std::fs::read_to_string(session_path)?;
        let session_data: SessionData = serde_json::from_str(&json)?;
        
        self.restore_session_data(session_data, app)?;
        
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        
        for entry in std::fs::read_dir(&self.session_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(name.to_string());
                }
            }
        }
        
        Ok(sessions)
    }

    pub fn delete_session(&self, name: &str) -> Result<()> {
        let session_path = self.session_dir.join(format!("{}.json", name));
        std::fs::remove_file(session_path)?;
        Ok(())
    }

    fn create_session_data(&self, app: &App) -> Result<SessionData> {
        let mut buffers = Vec::new();
        
        for buffer in &app.buffer_manager.buffers {
            buffers.push(SessionBuffer {
                id: buffer.id,
                file_path: buffer.file_path.clone(),
                content: buffer.content.get_content(),
                cursor_line: buffer.content.cursor_position().0,
                cursor_col: buffer.content.cursor_position().1,
                viewport_offset: buffer.content.get_viewport_offset(),
            });
        }
        
        let windows = self.convert_window_to_session(&app.window_manager.root);
        
        Ok(SessionData {
            buffers,
            windows,
            current_buffer: app.buffer_manager.current_buffer,
            current_window: app.window_manager.current_window_id,
        })
    }

    fn convert_window_to_session(&self, window: &Window) -> SessionWindow {
        match &window.split {
            Split::Leaf { buffer_id } => SessionWindow::Leaf { buffer_id: *buffer_id },
            Split::Horizontal { top, bottom, ratio } => SessionWindow::HSplit {
                top: Box::new(self.convert_window_to_session(top)),
                bottom: Box::new(self.convert_window_to_session(bottom)),
                ratio: *ratio,
            },
            Split::Vertical { left, right, ratio } => SessionWindow::VSplit {
                left: Box::new(self.convert_window_to_session(left)),
                right: Box::new(self.convert_window_to_session(right)),
                ratio: *ratio,
            },
        }
    }

    fn restore_session_data(&self, data: SessionData, app: &mut App) -> Result<()> {
        // Clear existing buffers
        app.buffer_manager.buffers.clear();
        app.buffer_manager.next_id = 0;
        
        // Restore buffers
        for session_buffer in data.buffers {
            let mut buffer = Buffer::new(session_buffer.id);
            buffer.file_path = session_buffer.file_path;
            buffer.content.set_content(session_buffer.content.clone());
            buffer.content.set_cursor_position(session_buffer.cursor_line, session_buffer.cursor_col);
            buffer.content.set_viewport_offset(session_buffer.viewport_offset);
            
            app.buffer_manager.buffers.push(buffer);
            if session_buffer.id >= app.buffer_manager.next_id {
                app.buffer_manager.next_id = session_buffer.id + 1;
            }
        }
        
        app.buffer_manager.current_buffer = data.current_buffer;
        
        // Restore windows
        app.window_manager.root = self.convert_session_to_window(&data.windows, 0);
        app.window_manager.current_window_id = data.current_window;
        
        // Sync current buffer
        app.sync_current_buffer();
        
        Ok(())
    }

    fn convert_session_to_window(&self, session_window: &SessionWindow, id: usize) -> Window {
        match session_window {
            SessionWindow::Leaf { buffer_id } => Window::new_leaf(id, *buffer_id),
            SessionWindow::HSplit { top, bottom, ratio } => Window {
                id,
                split: Split::Horizontal {
                    top: Box::new(self.convert_session_to_window(top, id)),
                    bottom: Box::new(self.convert_session_to_window(bottom, id + 1000)),
                    ratio: *ratio,
                },
                viewport_offset: 0,
                cursor_line: 0,
                cursor_col: 0,
            },
            SessionWindow::VSplit { left, right, ratio } => Window {
                id,
                split: Split::Vertical {
                    left: Box::new(self.convert_session_to_window(left, id)),
                    right: Box::new(self.convert_session_to_window(right, id + 1000)),
                    ratio: *ratio,
                },
                viewport_offset: 0,
                cursor_line: 0,
                cursor_col: 0,
            },
        }
    }
}

// プラグインシステム
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, app: &mut App) -> Result<()>;
    fn on_key(&mut self, app: &mut App, key: KeyEvent) -> Result<bool>;
    fn on_command(&mut self, app: &mut App, command: &str) -> Result<Option<String>>;
    fn on_save(&mut self, app: &mut App, path: &std::path::Path) -> Result<()>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

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
        self.plugins.iter().map(|p| (p.name(), p.version())).collect()
    }
}

impl WindowManager {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            root: Window::new_leaf(0, buffer_id),
            current_window_id: 0,
            next_window_id: 1,
        }
    }

    pub fn split_horizontal(&mut self, buffer_id: usize) -> usize {
        let new_id = self.next_window_id;
        self.next_window_id += 1;
        
        self.find_and_split_window(self.current_window_id, new_id, buffer_id, true);
        self.current_window_id = new_id;
        new_id
    }

    pub fn split_vertical(&mut self, buffer_id: usize) -> usize {
        let new_id = self.next_window_id;
        self.next_window_id += 1;
        
        self.find_and_split_window(self.current_window_id, new_id, buffer_id, false);
        self.current_window_id = new_id;
        new_id
    }

    fn find_and_split_window(&mut self, target_id: usize, new_id: usize, buffer_id: usize, horizontal: bool) {
        split_window_recursive(&mut self.root, target_id, new_id, buffer_id, horizontal);
    }

}

fn split_window_recursive(window: &mut Window, target_id: usize, new_id: usize, buffer_id: usize, horizontal: bool) -> bool {
        if window.id == target_id {
            if horizontal {
                window.split_horizontal(new_id, buffer_id);
            } else {
                window.split_vertical(new_id, buffer_id);
            }
            return true;
        }

        match &mut window.split {
            Split::Horizontal { top, bottom, .. } => {
                if split_window_recursive(top, target_id, new_id, buffer_id, horizontal) {
                    return true;
                }
                split_window_recursive(bottom, target_id, new_id, buffer_id, horizontal)
            }
            Split::Vertical { left, right, .. } => {
                if split_window_recursive(left, target_id, new_id, buffer_id, horizontal) {
                    return true;
                }
                split_window_recursive(right, target_id, new_id, buffer_id, horizontal)
            }
            Split::Leaf { .. } => false,
        }
}

impl WindowManager {
    pub fn close_window(&mut self, window_id: usize) -> Result<()> {
        if window_id == 0 {
            return Err(anyhow::anyhow!("Cannot close root window"));
        }
        // ウィンドウクローズロジック実装
        Ok(())
    }

    pub fn get_current_buffer_id(&self) -> usize {
        self.get_buffer_id(&self.root, self.current_window_id).unwrap_or(0)
    }

    fn get_buffer_id(&self, window: &Window, target_id: usize) -> Option<usize> {
        if window.id == target_id {
            match &window.split {
                Split::Leaf { buffer_id } => return Some(*buffer_id),
                _ => {}
            }
        }

        match &window.split {
            Split::Horizontal { top, bottom, .. } => {
                if let Some(id) = self.get_buffer_id(top, target_id) {
                    return Some(id);
                }
                self.get_buffer_id(bottom, target_id)
            }
            Split::Vertical { left, right, .. } => {
                if let Some(id) = self.get_buffer_id(left, target_id) {
                    return Some(id);
                }
                self.get_buffer_id(right, target_id)
            }
            Split::Leaf { buffer_id } if window.id == target_id => Some(*buffer_id),
            _ => None,
        }
    }

    pub fn next_window(&mut self) {
        let windows = self.collect_window_ids(&self.root);
        if let Some(pos) = windows.iter().position(|&id| id == self.current_window_id) {
            self.current_window_id = windows[(pos + 1) % windows.len()];
        }
    }

    pub fn prev_window(&mut self) {
        let windows = self.collect_window_ids(&self.root);
        if let Some(pos) = windows.iter().position(|&id| id == self.current_window_id) {
            let prev_pos = if pos == 0 { windows.len() - 1 } else { pos - 1 };
            self.current_window_id = windows[prev_pos];
        }
    }

    fn collect_window_ids(&self, window: &Window) -> Vec<usize> {
        match &window.split {
            Split::Horizontal { top, bottom, .. } => {
                let mut ids = self.collect_window_ids(top);
                ids.extend(self.collect_window_ids(bottom));
                ids
            }
            Split::Vertical { left, right, .. } => {
                let mut ids = self.collect_window_ids(left);
                ids.extend(self.collect_window_ids(right));
                ids
            }
            Split::Leaf { .. } => vec![window.id],
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
        let session_manager = SessionManager::new()?;
        
        Ok(Self {
            editor: Editor::new(),
            config,
            ui_state: UIState::new(),
            file_manager: FileManager::new(),
            command_processor: CommandProcessor::new(),
            buffer_manager,
            window_manager: WindowManager::new(initial_buffer_id),
            session_manager,
            plugin_manager: PluginManager::new(),
            last_key: None,
            macro_recording: false,
            macro_register: None,
            macro_keys: Vec::new(),
            macro_registers: std::collections::HashMap::new(),
        })
    }

    pub fn is_modified(&self) -> bool {
        self.editor.is_modified()
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
        if key.code != KeyCode::Char('d') && key.code != KeyCode::Char('q') && self.last_key.is_some() {
            self.last_key = None;
        }
        
        match key.code {
            // Vim-style movement
            KeyCode::Char('h') | KeyCode::Left => self.editor.move_cursor_left(),
            KeyCode::Char('j') | KeyCode::Down => self.editor.move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.editor.move_cursor_up(),
            KeyCode::Char('l') | KeyCode::Right => self.editor.move_cursor_right(),
            
            // Line movement
            KeyCode::Home => self.editor.move_to_line_start(),
            KeyCode::End => self.editor.move_to_line_end(),
            KeyCode::PageUp => self.editor.page_up(),
            KeyCode::PageDown => self.editor.page_down(),
            
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
                self.editor.start_visual_selection();
                self.ui_state.enter_visual_mode();
            }
            KeyCode::Char('V') => {
                // Visual line mode (treat as visual for now)
                self.editor.start_visual_selection();
                self.editor.move_to_line_start();
                self.ui_state.enter_visual_mode();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Visual block mode
                self.editor.start_visual_selection();
                self.ui_state.enter_visual_block_mode();
            }
            
            // Replace mode
            KeyCode::Char('R') => {
                self.ui_state.enter_replace_mode();
            }
            
            // Insert mode transitions
            KeyCode::Char('i') => self.ui_state.enter_insert_mode(),
            KeyCode::Char('a') => {
                self.editor.move_cursor_right();
                self.ui_state.enter_insert_mode();
            }
            KeyCode::Char('o') => {
                self.editor.move_to_line_end();
                self.editor.insert_newline();
                self.ui_state.enter_insert_mode();
            }
            KeyCode::Char('O') => {
                self.editor.move_to_line_start();
                self.editor.insert_newline();
                self.editor.move_cursor_up();
                self.ui_state.enter_insert_mode();
            }
            
            // Delete operations
            KeyCode::Char('x') => self.editor.delete_char_forward(),
            KeyCode::Char('d') => self.handle_delete_command(),
            
            // Paste
            KeyCode::Char('p') => {
                self.editor.paste();
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
            KeyCode::Char(c) => self.editor.insert_char(c),
            KeyCode::Enter => self.editor.insert_newline(),
            KeyCode::Backspace => self.editor.delete_char_backward(),
            KeyCode::Delete => self.editor.delete_char_forward(),
            KeyCode::Tab => self.editor.insert_tab(),
            
            // Cursor movement in insert mode
            KeyCode::Left => self.editor.move_cursor_left(),
            KeyCode::Right => self.editor.move_cursor_right(),
            KeyCode::Up => self.editor.move_cursor_up(),
            KeyCode::Down => self.editor.move_cursor_down(),
            
            _ => {}
        }
        Ok(())
    }
    
    fn handle_delete_command(&mut self) {
        // Vim dd command: delete line (second d press)
        if self.last_key == Some('d') {
            self.editor.delete_line();
            self.ui_state.set_success_message("Line deleted and yanked".to_string());
            self.last_key = None;
        } else {
            self.last_key = Some('d');
        }
    }
    
    fn handle_undo(&mut self) {
        if self.editor.undo() {
            self.ui_state.set_success_message("Undone".to_string());
        } else {
            self.ui_state.set_warning_message("Nothing to undo".to_string());
        }
    }
    
    fn handle_redo(&mut self) {
        if self.editor.redo() {
            self.ui_state.set_success_message("Redone".to_string());
        } else {
            self.ui_state.set_warning_message("Nothing to redo".to_string());
        }
    }

    fn handle_visual_mode_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit visual mode
            KeyCode::Esc => {
                self.editor.clear_visual_selection();
                self.ui_state.enter_normal_mode();
            }
            
            // Movement extends selection
            KeyCode::Char('h') | KeyCode::Left => self.editor.move_cursor_left(),
            KeyCode::Char('j') | KeyCode::Down => self.editor.move_cursor_down(),
            KeyCode::Char('k') | KeyCode::Up => self.editor.move_cursor_up(),
            KeyCode::Char('l') | KeyCode::Right => self.editor.move_cursor_right(),
            KeyCode::Home => self.editor.move_to_line_start(),
            KeyCode::End => self.editor.move_to_line_end(),
            
            // Operations on selection
            KeyCode::Char('d') | KeyCode::Char('x') => {
                self.editor.delete_selection();
                self.ui_state.enter_normal_mode();
                self.ui_state.set_success_message("Selection deleted and yanked".to_string());
            }
            KeyCode::Char('y') => {
                self.editor.yank_selection();
                self.editor.clear_visual_selection();
                self.ui_state.enter_normal_mode();
                self.ui_state.set_success_message("Selection yanked".to_string());
            }
            KeyCode::Char('c') => {
                self.editor.delete_selection();
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
                self.editor.replace_char(c);
            }
            KeyCode::Left => self.editor.move_cursor_left(),
            KeyCode::Right => self.editor.move_cursor_right(),
            KeyCode::Up => self.editor.move_cursor_up(),
            KeyCode::Down => self.editor.move_cursor_down(),
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
                
                match self.command_processor.execute_command(
                    &command, 
                    &mut self.editor, 
                    &mut self.file_manager,
                    &mut self.buffer_manager,
                    &mut self.window_manager,
                    &mut self.ui_state.should_quit
                ).await {
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
    pub fn set_status_message(&mut self, message: &str) {
        self.ui_state.set_status_message(message.to_string());
    }

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
                if let Err(e) = self.file_manager.save_file(&mut self.editor).await {
                    self.ui_state.set_error_message(format!("Error saving: {}", e));
                    self.ui_state.enter_normal_mode();
                } else {
                    self.ui_state.quit();
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.ui_state.quit();
            }
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
                self.ui_state.enter_normal_mode();
                self.ui_state.clear_status_message();
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
        self.ui_state.set_info_message(format!("Recording macro to register '{}'", register));
    }
    
    fn stop_macro_recording(&mut self) {
        if let Some(register) = self.macro_register {
            self.macro_registers.insert(register, self.macro_keys.clone());
            self.ui_state.set_success_message(format!("Macro recorded to register '{}'", register));
        }
        self.macro_recording = false;
        self.macro_register = None;
        self.macro_keys.clear();
    }
    
    fn play_macro(&mut self, register: char) {
        if let Some(keys) = self.macro_registers.get(&register).cloned() {
            self.ui_state.set_info_message(format!("Playing macro from register '{}'", register));
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
            self.ui_state.set_success_message("Macro playback complete".to_string());
        } else {
            self.ui_state.set_warning_message(format!("No macro in register '{}'", register));
        }
    }
    
    pub fn is_recording_macro(&self) -> bool {
        self.macro_recording
    }
    
    pub fn get_macro_register(&self) -> Option<char> {
        self.macro_register
    }

    fn sync_current_buffer(&mut self) {
        // Sync the current buffer based on the current window
        let buffer_id = self.window_manager.get_current_buffer_id();
        if let Ok(()) = self.buffer_manager.switch_to_buffer(buffer_id) {
            let buffer = self.buffer_manager.get_current();
            self.editor = buffer.content.clone();
            if let Some(path) = &buffer.file_path {
                self.file_manager.set_current_file(path.clone());
            }
        }
    }

    pub fn save_current_buffer_to_editor(&mut self) {
        // Save current editor state to the current buffer
        let buffer = self.buffer_manager.get_current_mut();
        buffer.content = self.editor.clone();
        buffer.modified = self.editor.is_modified();
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

        app.set_status_message("Test message");
        assert_eq!(app.status_message(), "Test message");
    }

    #[tokio::test]
    async fn test_command_execution_quit() {
        let mut app = App::new().await.unwrap();
        app.ui_state.set_mode(Mode::Command);
        app.ui_state.set_command_buffer("q".to_string());

        // Simulate command execution
        let command = app.ui_state.get_command_buffer().to_string();
        let result = app.command_processor.execute_command(
            &command,
            &mut app.editor,
            &mut app.file_manager,
            &mut app.buffer_manager,
            &mut app.window_manager,
            &mut app.ui_state.should_quit
        ).await;
        
        assert!(result.is_ok());
        assert!(app.should_quit()); // Should quit since no modifications
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let mut app = App::new().await.unwrap();
        app.editor.set_content("Hello World\nTest line".to_string());
        app.ui_state.set_mode(Mode::Command);
        app.ui_state.set_command_buffer("search World".to_string());
        
        // Simulate search command execution
        let command = app.ui_state.get_command_buffer().to_string();
        let result = app.command_processor.execute_command(
            &command,
            &mut app.editor,
            &mut app.file_manager,
            &mut app.buffer_manager,
            &mut app.window_manager,
            &mut app.ui_state.should_quit
        ).await;
        
        assert!(result.is_ok());
        
        // Check cursor moved to found position
        let (line, col) = app.editor.cursor_position();
        assert_eq!(line, 0);
        assert_eq!(col, 6); // "World" starts at column 6 in "Hello World"
    }

    #[tokio::test]
    async fn test_insert_mode_key_handling() {
        let mut app = App::new().await.unwrap();
        app.ui_state.set_mode(Mode::Insert);

        // Test character insertion
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('H'))).await;
        assert!(result.is_ok());
        assert_eq!(app.editor.get_content(), "H");

        // Test escape to normal mode
        let result = app.handle_editor_key(create_key_event(KeyCode::Esc)).await;
        assert!(result.is_ok());
        assert!(matches!(app.mode(), &Mode::Normal));
    }

    #[tokio::test]
    async fn test_normal_mode_vim_commands() {
        let mut app = App::new().await.unwrap();
        app.editor.set_content("Hello World".to_string());

        // Test 'i' to enter insert mode
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('i'))).await;
        assert!(result.is_ok());
        assert!(matches!(app.mode(), &Mode::Insert));
        assert_eq!(app.status_message(), "-- INSERT --");

        // Reset to normal mode
        app.ui_state.enter_normal_mode();

        // Test ':' to enter command mode
        let result = app.handle_editor_key(create_key_event(KeyCode::Char(':'))).await;
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
        app.editor.insert_char('a');
        app.editor.insert_char('b');
        assert_eq!(app.editor.get_content(), "ab");
        
        // Test undo through normal mode key
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('u'))).await;
        assert!(result.is_ok());
        assert_eq!(app.editor.get_content(), "a");
        
        // Test redo
        let redo_key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let result = app.handle_editor_key(redo_key).await;
        assert!(result.is_ok());
        assert_eq!(app.editor.get_content(), "ab");
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
        let mut app = App::new().await.unwrap();
        
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
        assert!(app.is_recording_macro());
        assert_eq!(app.get_macro_register(), Some('a'));
        
        // Record some keys
        app.macro_keys.push(create_key_event(KeyCode::Char('i')));
        app.macro_keys.push(create_key_event(KeyCode::Char('h')));
        app.macro_keys.push(create_key_event(KeyCode::Char('i')));
        app.macro_keys.push(create_key_event(KeyCode::Esc));
        
        // Stop recording
        app.stop_macro_recording();
        assert!(!app.is_recording_macro());
        assert_eq!(app.get_macro_register(), None);
        
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
        app.editor.set_content("Hello World".to_string());
        
        // Play macro
        app.play_macro('b');
        
        // Check that 'x' command was executed (delete char)
        assert_eq!(app.editor.get_content(), "ello World");
        
        // Try playing non-existent macro
        app.play_macro('z');
        assert!(app.ui_state.get_status_message().contains("No macro"));
    }

    #[tokio::test]
    async fn test_dd_command() {
        let mut app = App::new().await.unwrap();
        app.editor.set_content("Line 1
Line 2
Line 3".to_string());
        
        // First 'd' should not delete
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('d'))).await;
        assert!(result.is_ok());
        assert_eq!(app.editor.get_content(), "Line 1
Line 2
Line 3");
        assert_eq!(app.last_key, Some('d'));
        
        // Second 'd' should delete the line
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('d'))).await;
        assert!(result.is_ok());
        assert_eq!(app.editor.get_content(), "Line 2
Line 3");
        assert_eq!(app.last_key, None);
        
        // Test paste after dd (paste puts the line at cursor position)
        let result = app.handle_editor_key(create_key_event(KeyCode::Char('p'))).await;
        assert!(result.is_ok());
        // The pasted line should be inserted at current position
        let content = app.editor.get_content();
        assert!(content.contains("Line 1")); // Original line 1 should still be in content
    }
}