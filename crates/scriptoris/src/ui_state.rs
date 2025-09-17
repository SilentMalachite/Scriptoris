use crate::app::Mode;
use crate::status_manager::StatusManager;

#[derive(Clone)]
pub struct UIState {
    pub mode: Mode,
    pub status_message: String, // Keep for backward compatibility
    pub status_manager: StatusManager,
    pub show_help: bool,
    pub command_buffer: String,
    pub should_quit: bool,
    // Command history
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            status_message: String::new(),
            status_manager: StatusManager::new(),
            show_help: false,
            command_buffer: String::new(),
            should_quit: false,
            command_history: Vec::new(),
            history_index: None,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn get_status_message(&self) -> &str {
        &self.status_message
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.mode = if self.show_help {
            Mode::Help
        } else {
            Mode::Normal
        };
    }

    pub fn is_help_shown(&self) -> bool {
        self.show_help
    }

    pub fn hide_help(&mut self) {
        self.show_help = false;
        if matches!(self.mode, Mode::Help) {
            self.mode = Mode::Normal;
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_buffer.clear();
        self.status_message = ":".to_string();
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_buffer = "search ".to_string();
        self.status_message = "/".to_string();
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
        self.status_manager
            .set_mode_message("-- INSERT --".to_string());
        self.status_message = "-- INSERT --".to_string();
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.status_manager.set_mode_message("".to_string());
        self.status_message.clear();
    }

    pub fn enter_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        self.status_manager
            .set_mode_message("-- VISUAL --".to_string());
        self.status_message = "-- VISUAL --".to_string();
    }

    pub fn enter_visual_block_mode(&mut self) {
        self.mode = Mode::VisualBlock;
        self.status_manager
            .set_mode_message("-- VISUAL BLOCK --".to_string());
        self.status_message = "-- VISUAL BLOCK --".to_string();
    }

    pub fn enter_replace_mode(&mut self) {
        self.mode = Mode::Replace;
        self.status_manager
            .set_mode_message("-- REPLACE --".to_string());
        self.status_message = "-- REPLACE --".to_string();
    }

    pub fn get_command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn set_command_buffer(&mut self, buffer: String) {
        self.command_buffer = buffer;
    }

    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    pub fn push_to_command_buffer(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    pub fn pop_from_command_buffer(&mut self) {
        self.command_buffer.pop();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    // Enhanced status methods using StatusManager
    pub fn set_info_message(&mut self, message: String) {
        self.status_message = message.clone();
        self.status_manager.set_info(message); // Keep backward compatibility
    }

    pub fn set_success_message(&mut self, message: String) {
        self.status_message = message.clone();
        self.status_manager.set_success(message);
    }

    pub fn set_warning_message(&mut self, message: String) {
        self.status_message = message.clone();
        self.status_manager.set_warning(message);
    }

    pub fn set_error_message(&mut self, message: String) {
        self.status_message = message.clone();
        self.status_manager.set_error(message);
    }

    pub fn update_status(&mut self) {
        self.status_manager.update();

        // Update backward compatible status_message
        if let Some(current) = self.status_manager.get_current_message() {
            self.status_message.clear();
            self.status_message.push_str(&current.content);
        } else if self.status_manager.get_mode_message().is_empty() {
            self.status_message.clear();
        }
    }

    // Enhanced status methods with duration control

    // Command history methods
    pub fn add_to_history(&mut self, command: String) {
        // Don't add empty commands or duplicates of the last command
        if !command.is_empty() && self.command_history.last() != Some(&command) {
            self.command_history.push(command);
            // Limit history size
            if self.command_history.len() > 100 {
                self.command_history.remove(0);
            }
        }
        self.history_index = None;
    }

    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => self.command_history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };

        self.history_index = Some(new_index);
        self.command_buffer = self.command_history[new_index].clone();
    }

    pub fn history_down(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {}
            Some(i) if i >= self.command_history.len() - 1 => {
                self.history_index = None;
                self.command_buffer.clear();
            }
            Some(i) => {
                let new_index = i + 1;
                self.history_index = Some(new_index);
                self.command_buffer = self.command_history[new_index].clone();
            }
        }
    }

    pub fn get_command_suggestions(&self, prefix: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Built-in commands
        let commands = vec![
            "w", "q", "wq", "q!", "e", "help", "set", "vsplit", "split", "tabnew", "tabnext",
            "tabprev", "buffer", "bnext", "bprev",
        ];

        for cmd in commands {
            if cmd.starts_with(prefix) {
                suggestions.push(cmd.to_string());
            }
        }

        // Add from history
        for cmd in &self.command_history {
            if cmd.starts_with(prefix) && !suggestions.contains(cmd) {
                suggestions.push(cmd.clone());
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_creation() {
        let state = UIState::new();
        assert!(matches!(state.mode, Mode::Normal));
        assert_eq!(state.status_message, "");
        assert!(!state.show_help);
        assert_eq!(state.command_buffer, "");
        assert!(!state.should_quit);
    }

    #[test]
    fn test_mode_transitions() {
        let mut state = UIState::new();

        // Test insert mode
        state.enter_insert_mode();
        assert!(matches!(state.mode, Mode::Insert));
        assert_eq!(state.status_message, "-- INSERT --");

        // Test normal mode
        state.enter_normal_mode();
        assert!(matches!(state.mode, Mode::Normal));
        assert_eq!(state.status_message, "");

        // Test command mode
        state.enter_command_mode();
        assert!(matches!(state.mode, Mode::Command));
        assert_eq!(state.status_message, ":");
        assert_eq!(state.command_buffer, "");

        // Test search mode
        state.enter_search_mode();
        assert!(matches!(state.mode, Mode::Command));
        assert_eq!(state.status_message, "/");
        assert_eq!(state.command_buffer, "search ");
    }

    #[test]
    fn test_help_toggle() {
        let mut state = UIState::new();

        assert!(!state.is_help_shown());

        state.toggle_help();
        assert!(state.is_help_shown());
        assert!(matches!(state.mode, Mode::Help));

        state.toggle_help();
        assert!(!state.is_help_shown());
        assert!(matches!(state.mode, Mode::Normal));
    }

    #[test]
    fn test_command_buffer_operations() {
        let mut state = UIState::new();

        state.push_to_command_buffer('w');
        state.push_to_command_buffer('q');
        assert_eq!(state.command_buffer, "wq");

        state.pop_from_command_buffer();
        assert_eq!(state.command_buffer, "w");

        state.clear_command_buffer();
        assert_eq!(state.command_buffer, "");
    }

    #[test]
    fn test_quit_operations() {
        let mut state = UIState::new();

        assert!(!state.should_quit());

        state.quit();
        assert!(state.should_quit());

        state.should_quit = false;
        assert!(!state.should_quit());
    }

    #[test]
    fn test_status_message() {
        let mut state = UIState::new();

        state.set_info_message("Test message".to_string());
        assert_eq!(state.get_status_message(), "Test message");

        state.status_message.clear();
        assert_eq!(state.get_status_message(), "");
    }
}
