use ropey::Rope;
use std::cmp;

use crate::text_width::{EmojiWidth, TextWidthCalculator};

#[derive(Clone)]
pub struct Editor {
    rope: Rope,
    cursor_line: usize,
    cursor_col: usize,
    viewport_offset: usize,
    viewport_height: usize,
    modified: bool,
    clipboard: String,
    // Undo/Redo support
    history: Vec<EditorState>,
    history_index: usize,
    // Visual mode selection
    visual_start_line: Option<usize>,
    visual_start_col: Option<usize>,
    // Text width calculator for accurate cursor positioning
    text_calculator: TextWidthCalculator,
    // Tab configuration
    tab_size: usize,
    use_spaces: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
struct EditorState {
    content: String,
    cursor_line: usize,
    cursor_col: usize,
    visual_start_line: Option<usize>,
    visual_start_col: Option<usize>,
}

impl Editor {
    pub fn new() -> Self {
        let initial_state = EditorState {
            content: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            visual_start_line: None,
            visual_start_col: None,
        };

        // Configure text calculator for cross-platform compatibility
        let text_calculator = TextWidthCalculator::new()
            .east_asian_aware(true)
            .emoji_width(EmojiWidth::Standard);

        Self {
            rope: Rope::new(),
            cursor_line: 0,
            cursor_col: 0,
            viewport_offset: 0,
            viewport_height: 24, // Default, will be updated
            modified: false,
            clipboard: String::new(),
            history: vec![initial_state],
            history_index: 0,
            visual_start_line: None,
            visual_start_col: None,
            text_calculator,
            tab_size: 4,
            use_spaces: true,
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.rope = Rope::from_str(&content);
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.viewport_offset = 0;
        self.modified = false;
        self.visual_start_line = None;
        self.visual_start_col = None;

        // Reset history with new content
        let initial_state = EditorState {
            content,
            cursor_line: 0,
            cursor_col: 0,
            visual_start_line: None,
            visual_start_col: None,
        };
        self.history = vec![initial_state];
        self.history_index = 0;
    }

    pub fn get_content(&self) -> String {
        self.rope.to_string()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_line, self.cursor_col)
    }

    pub fn set_cursor_position(&mut self, line: usize, col: usize) {
        // Ensure line is within bounds
        let max_line = self.rope.len_lines().saturating_sub(1);
        self.cursor_line = line.min(max_line);

        // Ensure column is within bounds for the current line
        if let Some(line_content) = self.rope.get_line(self.cursor_line) {
            let max_col = line_content.len_chars().saturating_sub(1);
            self.cursor_col = col.min(max_col);
        } else {
            self.cursor_col = 0;
        }

        self.adjust_viewport();
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    pub fn set_viewport_offset(&mut self, offset: usize) {
        let max_offset = self.rope.len_lines().saturating_sub(self.viewport_height);
        self.viewport_offset = offset.min(max_offset);
    }

    pub fn get_viewport_offset(&self) -> usize {
        self.viewport_offset
    }

    pub fn get_viewport_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let end_line = cmp::min(
            self.viewport_offset + self.viewport_height,
            self.rope.len_lines(),
        );

        for i in self.viewport_offset..end_line {
            if let Some(line) = self.rope.get_line(i) {
                lines.push(line.to_string());
            }
        }

        lines
    }

    pub fn insert_char(&mut self, c: char) {
        // Handle memory constraints gracefully
        if self.rope.len_chars() > 1_000_000 {
            // 1 million characters
            log::warn!("Document size approaching limit, insert may be slow");
        }

        let char_idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);

        // Insert character
        self.rope.insert_char(char_idx, c);

        // Move cursor forward using text_calculator for proper width
        self.cursor_col += self.text_calculator.grapheme_width(&c.to_string());
        self.modified = true;
        self.save_state();
    }

    pub fn insert_newline(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);
        self.rope.insert_char(char_idx, '\n');
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.modified = true;
        self.adjust_viewport();
        self.save_state();
    }

    pub fn insert_tab(&mut self) {
        if self.use_spaces {
            // Insert spaces according to tab_size
            for _ in 0..self.tab_size {
                self.insert_char(' ');
            }
        } else {
            // Insert actual tab character
            self.insert_char('\t');
        }
    }

    /// Set tab configuration
    pub fn set_tab_config(&mut self, tab_size: usize, use_spaces: bool) {
        self.tab_size = tab_size;
        self.use_spaces = use_spaces;
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            let char_idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);
            self.rope.remove(char_idx..char_idx + 1);
            self.modified = true;
            self.save_state();
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            if let Some(line) = self.rope.get_line(self.cursor_line) {
                self.cursor_col = line.len_chars().saturating_sub(1);
            }
            let char_idx = self.line_col_to_char_idx(self.cursor_line + 1, 0) - 1;
            self.rope.remove(char_idx..char_idx + 1);
            self.modified = true;
            self.adjust_viewport();
            self.save_state();
        }
    }

    pub fn delete_char_forward(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);
        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
            self.modified = true;
            self.save_state();
        }
    }

    pub fn delete_line(&mut self) {
        if let Some(line) = self.rope.get_line(self.cursor_line) {
            self.clipboard = line.to_string();
            let start_idx = self.rope.line_to_char(self.cursor_line);
            let end_idx = if self.cursor_line + 1 < self.rope.len_lines() {
                self.rope.line_to_char(self.cursor_line + 1)
            } else {
                self.rope.len_chars()
            };
            self.rope.remove(start_idx..end_idx);
            self.cursor_col = 0;
            self.modified = true;
            self.save_state();
        }
    }

    pub fn yank_line(&mut self) {
        if let Some(line) = self.rope.get_line(self.cursor_line) {
            self.clipboard = line.to_string();
            // Don't modify the document, just copy to clipboard
        }
    }

    pub fn paste(&mut self) {
        if !self.clipboard.is_empty() {
            let char_idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);
            self.rope.insert(char_idx, &self.clipboard);
            self.modified = true;
            self.save_state();
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.adjust_cursor_col();
            self.adjust_viewport();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.rope.len_lines() {
            self.cursor_line += 1;
            self.adjust_cursor_col();
            self.adjust_viewport();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            if let Some(line) = self.rope.get_line(self.cursor_line) {
                self.cursor_col = line.len_chars().saturating_sub(1);
            }
            self.adjust_viewport();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(line) = self.rope.get_line(self.cursor_line) {
            let line_len = line.len_chars().saturating_sub(1);
            if self.cursor_col < line_len {
                self.cursor_col += 1;
            } else if self.cursor_line + 1 < self.rope.len_lines() {
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.adjust_viewport();
            }
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        if let Some(line) = self.rope.get_line(self.cursor_line) {
            self.cursor_col = line.len_chars().saturating_sub(1);
        }
    }

    pub fn page_up(&mut self) {
        let new_line = self.cursor_line.saturating_sub(self.viewport_height);
        self.cursor_line = new_line;
        self.viewport_offset = self.viewport_offset.saturating_sub(self.viewport_height);
        self.adjust_cursor_col();
    }

    pub fn page_down(&mut self) {
        let max_line = self.rope.len_lines().saturating_sub(1);
        let new_line = cmp::min(self.cursor_line + self.viewport_height, max_line);
        self.cursor_line = new_line;
        self.viewport_offset = cmp::min(
            self.viewport_offset + self.viewport_height,
            max_line.saturating_sub(self.viewport_height),
        );
        self.adjust_cursor_col();
    }

    fn line_col_to_char_idx(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_char(line);
        if let Some(line_text) = self.rope.get_line(line) {
            // text_calculatorを使用して表示カラム位置から文字インデックスに変換
            let char_offset = self
                .text_calculator
                .col_to_char_index(&line_text.to_string(), col);
            line_start + char_offset
        } else {
            line_start + col
        }
    }

    fn char_idx_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let char_offset = char_idx - line_start;

        if let Some(line_text) = self.rope.get_line(line) {
            // text_calculatorを使用して文字インデックスから表示カラム位置に変換
            let col = self
                .text_calculator
                .char_index_to_col(&line_text.to_string(), char_offset);
            (line, col)
        } else {
            (line, char_offset)
        }
    }

    fn adjust_cursor_col(&mut self) {
        if let Some(line) = self.rope.get_line(self.cursor_line) {
            // text_calculatorを使用して行の表示幅を取得
            let line_display_width = self.text_calculator.str_width(&line.to_string());
            self.cursor_col = cmp::min(self.cursor_col, line_display_width);
        }
    }

    fn adjust_viewport(&mut self) {
        if self.cursor_line < self.viewport_offset {
            self.viewport_offset = self.cursor_line;
        } else if self.cursor_line >= self.viewport_offset + self.viewport_height {
            self.viewport_offset = self.cursor_line.saturating_sub(self.viewport_height - 1);
        }
    }

    pub fn search(&mut self, query: &str) {
        let content = self.rope.to_string();
        let current_pos = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);

        if let Some(pos) = content[current_pos..].find(query) {
            let found_pos = current_pos + pos;
            let (line, col) = self.char_idx_to_line_col(found_pos);
            self.cursor_line = line;
            self.cursor_col = col;
            self.adjust_viewport();
        }
    }

    pub fn save_state(&mut self) {
        let current_state = EditorState {
            content: self.rope.to_string(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
            visual_start_line: self.visual_start_line,
            visual_start_col: self.visual_start_col,
        };

        // Don't save if the content hasn't changed from current history state
        if let Some(last_state) = self.history.get(self.history_index) {
            if last_state.content == current_state.content {
                return;
            }
        }

        // Remove any states after current index (if we're not at the end)
        self.history.truncate(self.history_index + 1);

        // Add new state
        self.history.push(current_state);
        self.history_index += 1;

        // Limit history size to prevent memory issues
        if self.history.len() > 100 {
            self.history.remove(0);
            self.history_index -= 1;
        }
    }

    pub fn undo(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            let state = &self.history[self.history_index];
            self.rope = Rope::from_str(&state.content);
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
            self.adjust_viewport();
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            let state = &self.history[self.history_index];
            self.rope = Rope::from_str(&state.content);
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
            self.adjust_viewport();
            self.modified = true;
            true
        } else {
            false
        }
    }

    // Visual mode selection methods
    pub fn start_visual_selection(&mut self) {
        self.visual_start_line = Some(self.cursor_line);
        self.visual_start_col = Some(self.cursor_col);
    }

    pub fn clear_visual_selection(&mut self) {
        self.visual_start_line = None;
        self.visual_start_col = None;
    }

    pub fn get_visual_selection(&self) -> Option<(usize, usize, usize, usize)> {
        if let (Some(start_line), Some(start_col)) = (self.visual_start_line, self.visual_start_col)
        {
            let (end_line, end_col) = (self.cursor_line, self.cursor_col);

            // Ensure start is before end
            if start_line < end_line || (start_line == end_line && start_col <= end_col) {
                Some((start_line, start_col, end_line, end_col))
            } else {
                Some((end_line, end_col, start_line, start_col))
            }
        } else {
            None
        }
    }

    pub fn get_selected_text(&self) -> String {
        if let Some((start_line, start_col, end_line, end_col)) = self.get_visual_selection() {
            let start_idx = self.line_col_to_char_idx(start_line, start_col);
            let end_idx = self.line_col_to_char_idx(end_line, end_col);
            self.rope.slice(start_idx..end_idx).to_string()
        } else {
            String::new()
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some((start_line, start_col, end_line, end_col)) = self.get_visual_selection() {
            let start_idx = self.line_col_to_char_idx(start_line, start_col);
            let end_idx = self.line_col_to_char_idx(end_line, end_col);

            // Save deleted text to clipboard
            self.clipboard = self.rope.slice(start_idx..end_idx).to_string();

            self.rope.remove(start_idx..end_idx);
            self.cursor_line = start_line;
            self.cursor_col = start_col;
            self.clear_visual_selection();
            self.adjust_cursor_col();
            self.save_state();
            self.modified = true;
        }
    }

    pub fn yank_selection(&mut self) {
        self.clipboard = self.get_selected_text();
    }

    // Replace mode methods
    pub fn replace_char(&mut self, c: char) {
        let idx = self.line_col_to_char_idx(self.cursor_line, self.cursor_col);
        if idx < self.rope.len_chars() {
            self.rope.remove(idx..idx + 1);
            self.rope.insert_char(idx, c);
            self.move_cursor_right();
            self.save_state();
            self.modified = true;
        }
    }
}
impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = Editor::new();
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 0);
        assert_eq!(editor.line_count(), 1); // Empty editor has one empty line
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_text_insertion() {
        let mut editor = Editor::new();
        editor.insert_char('H');
        editor.insert_char('i');

        assert_eq!(editor.get_content(), "Hi");
        assert_eq!(editor.cursor_col, 2);
        assert!(editor.is_modified());
    }

    #[test]
    fn test_newline_insertion() {
        let mut editor = Editor::new();
        editor.insert_char('H');
        editor.insert_char('i');
        editor.insert_newline();
        editor.insert_char('!');

        assert_eq!(editor.get_content(), "Hi\n!");
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 1);
        assert_eq!(editor.line_count(), 2);
    }

    #[test]
    fn test_backspace() {
        let mut editor = Editor::new();
        editor.insert_char('H');
        editor.insert_char('i');
        editor.delete_char_backward();

        assert_eq!(editor.get_content(), "H");
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = Editor::new();
        editor.set_content("Hello\nWorld".to_string());

        // Test right movement
        editor.move_cursor_right();
        assert_eq!(editor.cursor_col, 1);

        // Test down movement
        editor.move_cursor_down();
        assert_eq!(editor.cursor_line, 1);

        // Test left movement
        editor.move_cursor_left();
        assert_eq!(editor.cursor_col, 0);

        // Test up movement
        editor.move_cursor_up();
        assert_eq!(editor.cursor_line, 0);
    }

    #[test]
    fn test_search_functionality() {
        let mut editor = Editor::new();
        editor.set_content("Hello World\nHi there".to_string());

        // Search for "World" - should move cursor to line 0, col 6
        editor.search("World");
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 6);

        // Search for "Hi" - should move cursor to line 1, col 0
        editor.search("Hi");
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_line_operations() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2\nLine 3".to_string());
        editor.cursor_line = 1; // Move to second line

        // Test line deletion
        editor.delete_line();
        assert_eq!(editor.get_content(), "Line 1\nLine 3");
        assert_eq!(editor.line_count(), 2);

        // Test paste
        editor.paste();
        assert_eq!(editor.get_content(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_modified_state() {
        let mut editor = Editor::new();
        assert!(!editor.is_modified());

        editor.insert_char('a');
        assert!(editor.is_modified());

        editor.mark_saved();
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_content_setting() {
        let mut editor = Editor::new();
        let test_content = "This is a test\nWith multiple lines\nAnd more content";

        editor.set_content(test_content.to_string());
        assert_eq!(editor.get_content(), test_content);
        assert_eq!(editor.line_count(), 3);
        assert!(!editor.is_modified()); // set_content should not mark as modified
    }

    #[test]
    fn test_undo_redo_functionality() {
        let mut editor = Editor::new();

        // Initial state should have no undo/redo
        assert!(!editor.undo());
        assert!(!editor.redo());

        // Insert some text
        editor.insert_char('H');
        editor.insert_char('i');
        assert_eq!(editor.get_content(), "Hi");

        // Test undo
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "H");

        // Test redo
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "Hi");

        // Test multiple operations and undo
        editor.insert_char('!');
        assert_eq!(editor.get_content(), "Hi!");

        // Undo should restore previous state
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "Hi");

        // Undo again
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "H");

        // Try to undo beyond history
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        assert!(!editor.undo()); // Should fail - no more history
    }

    #[test]
    fn test_undo_redo_with_line_operations() {
        let mut editor = Editor::new();

        editor.insert_char('L');
        editor.insert_char('i');
        editor.insert_char('n');
        editor.insert_char('e');
        editor.insert_char(' ');
        editor.insert_char('1');
        editor.insert_newline();
        editor.insert_char('L');
        editor.insert_char('i');
        editor.insert_char('n');
        editor.insert_char('e');
        editor.insert_char(' ');
        editor.insert_char('2');

        assert_eq!(editor.get_content(), "Line 1\nLine 2");

        // Delete the second line
        editor.cursor_line = 1;
        editor.delete_line();
        assert_eq!(editor.get_content(), "Line 1\n");

        // Undo should restore the line
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "Line 1\nLine 2");
    }

    #[test]
    fn test_visual_mode_selection() {
        let mut editor = Editor::new();
        editor.set_content("Hello World\nTest Line\nThird Line".to_string());

        // Start visual selection at beginning
        editor.start_visual_selection();
        assert!(editor.visual_start_line.is_some());
        assert_eq!(editor.visual_start_line, Some(0));
        assert_eq!(editor.visual_start_col, Some(0));

        // Move cursor to select text
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right(); // Select "Hello"

        let selected = editor.get_selected_text();
        assert_eq!(selected, "Hello");

        // Clear selection
        editor.clear_visual_selection();
        assert!(editor.visual_start_line.is_none());
    }

    #[test]
    fn test_visual_selection_multiline() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2\nLine 3".to_string());

        // Start selection at Line 1
        editor.start_visual_selection();

        // Move to Line 2
        editor.move_cursor_down();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();

        let selected = editor.get_selected_text();
        assert!(selected.contains("Line 1"));
        assert!(selected.contains("Line"));
    }

    #[test]
    fn test_replace_char() {
        let mut editor = Editor::new();
        editor.set_content("Hello World".to_string());

        // Replace 'H' with 'J'
        editor.replace_char('J');
        assert_eq!(editor.get_content(), "Jello World");
        assert_eq!(editor.cursor_col, 1);

        // Replace 'e' with 'i'
        editor.replace_char('i');
        assert_eq!(editor.get_content(), "Jillo World");
    }

    #[test]
    fn test_delete_selection() {
        let mut editor = Editor::new();
        editor.set_content("Hello World".to_string());

        // Select "Hello"
        editor.start_visual_selection();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();

        editor.delete_selection();
        assert_eq!(editor.get_content(), " World");
        assert!(editor.visual_start_line.is_none());

        // Check clipboard
        assert_eq!(editor.clipboard, "Hello");
    }

    #[test]
    fn test_yank_selection() {
        let mut editor = Editor::new();
        editor.set_content("Copy this text".to_string());

        // Select "Copy"
        editor.start_visual_selection();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();

        editor.yank_selection();
        assert_eq!(editor.clipboard, "Copy");

        // Text should still be there
        assert_eq!(editor.get_content(), "Copy this text");
    }

    #[test]
    fn test_history_limit() {
        let mut editor = Editor::new();

        // Insert more than 100 characters to test history limit
        for i in 0..110 {
            editor.insert_char((b'a' + (i % 26) as u8) as char);
        }

        // History should be limited, but we should still be able to undo some operations
        assert!(editor.undo());
        assert!(editor.history.len() <= 100);
    }

    #[test]
    fn test_set_content_resets_history() {
        let mut editor = Editor::new();

        // Add some content and changes
        editor.insert_char('H');
        editor.insert_char('i');
        assert!(editor.undo()); // Should work

        // Set new content should reset history
        editor.set_content("New content".to_string());
        assert_eq!(editor.get_content(), "New content");

        // Should not be able to undo to previous content
        assert!(!editor.undo());

        // But can undo changes made after set_content
        editor.insert_char('!');
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "New content");
    }

    #[test]
    fn test_japanese_text_insertion() {
        let mut editor = Editor::new();
        editor.set_content("こんにちは".to_string());

        assert_eq!(editor.get_content(), "こんにちは");
        assert_eq!(editor.line_count(), 1);
    }

    #[test]
    fn test_japanese_text_deletion() {
        let mut editor = Editor::new();
        editor.set_content("こんにちは世界".to_string());

        // Verify Japanese text is handled
        assert_eq!(editor.line_count(), 1);
        let content = editor.get_content();
        assert!(content.contains("こんにちは"));
        assert!(content.contains("世界"));
    }

    #[test]
    fn test_mixed_text_editing() {
        let mut editor = Editor::new();

        // Mixed Japanese and English
        let content = "Hello世界Test";
        editor.set_content(content.to_string());

        // Test cursor movement
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();
        editor.move_cursor_right();

        // Insert text at cursor position
        editor.insert_char('!');

        assert!(editor.get_content().contains("!"));
    }

    #[test]
    fn test_emoji_editing() {
        let mut editor = Editor::new();
        editor.set_content("😀🎉".to_string());

        assert_eq!(editor.get_content(), "😀🎉");
        assert_eq!(editor.line_count(), 1);
    }

    #[test]
    fn test_fullwidth_characters() {
        let mut editor = Editor::new();
        editor.set_content("全角文字".to_string());

        // Fullwidth characters should be handled correctly
        assert_eq!(editor.line_count(), 1);
        assert_eq!(editor.get_content(), "全角文字");
    }
}
