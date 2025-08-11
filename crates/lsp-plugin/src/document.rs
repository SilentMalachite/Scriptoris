use lsp_types::*;
use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub language_id: String,
    pub version: i32,
    pub content: Rope,
}

impl Document {
    pub fn new(uri: Url, content: String, language_id: String, version: i32) -> Self {
        Self {
            uri,
            language_id,
            version,
            content: Rope::from_str(&content),
        }
    }

    pub fn update(&mut self, content: String, version: i32) {
        self.content = Rope::from_str(&content);
        self.version = version;
    }

    pub fn get_line(&self, line: usize) -> Option<String> {
        if line < self.content.len_lines() {
            Some(self.content.line(line).to_string())
        } else {
            None
        }
    }

    pub fn get_position_offset(&self, position: Position) -> Option<usize> {
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;
        
        if line_idx >= self.content.len_lines() {
            return None;
        }
        
        let line_start = self.content.line_to_char(line_idx);
        let line = self.content.line(line_idx);
        let line_str = line.as_str().unwrap_or("");
        
        // Convert UTF-16 code units to byte offset
        let mut utf16_pos = 0;
        let mut byte_pos = 0;
        
        for ch in line_str.chars() {
            if utf16_pos >= char_idx {
                break;
            }
            utf16_pos += ch.len_utf16();
            byte_pos += ch.len_utf8();
        }
        
        Some(line_start + byte_pos)
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self.content.char_to_line(offset);
        let line_start = self.content.line_to_char(line);
        let column = offset - line_start;
        
        // Convert byte offset to UTF-16 code units
        let line_str = self.content.line(line).as_str().unwrap_or("");
        let mut utf16_col = 0;
        let mut byte_count = 0;
        
        for ch in line_str.chars() {
            if byte_count >= column {
                break;
            }
            byte_count += ch.len_utf8();
            utf16_col += ch.len_utf16();
        }
        
        Position {
            line: line as u32,
            character: utf16_col as u32,
        }
    }

    pub fn apply_text_edits(&mut self, edits: Vec<TextEdit>) -> String {
        // Sort edits by position (reverse order to apply from end to start)
        let mut sorted_edits = edits;
        sorted_edits.sort_by(|a, b| {
            b.range.start.line.cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });
        
        for edit in sorted_edits {
            self.apply_text_edit(edit);
        }
        
        self.version += 1;
        self.content.to_string()
    }

    fn apply_text_edit(&mut self, edit: TextEdit) {
        if let (Some(start_offset), Some(end_offset)) = (
            self.get_position_offset(edit.range.start),
            self.get_position_offset(edit.range.end),
        ) {
            self.content.remove(start_offset..end_offset);
            self.content.insert(start_offset, &edit.new_text);
        }
    }

    pub fn get_word_at_position(&self, position: Position) -> Option<String> {
        let line = self.get_line(position.line as usize)?;
        let char_idx = position.character as usize;
        
        // Find word boundaries
        let chars: Vec<char> = line.chars().collect();
        if char_idx >= chars.len() {
            return None;
        }
        
        // Find start of word
        let mut start = char_idx;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        // Find end of word
        let mut end = char_idx;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start == end {
            None
        } else {
            Some(chars[start..end].iter().collect())
        }
    }
}