use lsp_types::*;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub content: String,
    pub language_id: String,
    pub version: i32,
}

impl Document {
    pub fn new(uri: Url, content: String, language_id: String, version: i32) -> Self {
        Self {
            uri,
            content,
            language_id,
            version,
        }
    }

    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
    }

    pub fn get_line(&self, line: usize) -> Option<String> {
        self.content.lines().nth(line).map(|s| s.to_string())
    }

    pub fn get_position_offset(&self, position: Position) -> Option<usize> {
        let line_idx = position.line as usize;
        let utf16_char_idx = position.character as usize;

        let lines: Vec<&str> = self.content.lines().collect();
        if line_idx >= lines.len() {
            return None;
        }

        let mut byte_offset = 0;
        for (i, line) in lines.iter().enumerate() {
            if i == line_idx {
                // Calculate offset within this line using grapheme clusters
                let mut utf16_count = 0;
                let mut byte_count = 0;
                
                for grapheme in line.graphemes(true) {
                    if utf16_count >= utf16_char_idx {
                        return Some(byte_offset + byte_count);
                    }
                    // Count UTF-16 code units for this grapheme
                    let grapheme_utf16_len: usize = grapheme.chars()
                        .map(|c| c.len_utf16())
                        .sum();
                    utf16_count += grapheme_utf16_len;
                    byte_count += grapheme.len();
                }
                // If character index is beyond line end, return line end
                return Some(byte_offset + line.len());
            }
            byte_offset += line.len() + 1; // +1 for newline
        }

        None
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let mut byte_offset = 0;
        let mut line_num = 0;

        for line in self.content.lines() {
            let line_len = line.len();
            if byte_offset + line_len >= offset {
                // Position is within this line
                let remaining = offset.saturating_sub(byte_offset);

                // Convert byte offset to UTF-16 code units using grapheme clusters
                let mut utf16_count = 0;
                let mut byte_count = 0;
                
                for grapheme in line.graphemes(true) {
                    if byte_count >= remaining {
                        break;
                    }
                    // Count UTF-16 code units for this grapheme
                    let grapheme_utf16_len: usize = grapheme.chars()
                        .map(|c| c.len_utf16())
                        .sum();
                    utf16_count += grapheme_utf16_len;
                    byte_count += grapheme.len();
                }

                return Position {
                    line: line_num as u32,
                    character: utf16_count as u32,
                };
            }
            byte_offset += line_len + 1; // +1 for newline
            line_num += 1;
        }

        // If offset is beyond content, return end position
        Position {
            line: self.content.lines().count() as u32,
            character: 0,
        }
    }

    pub fn apply_text_edits(&mut self, edits: Vec<TextEdit>) -> String {
        // Sort edits by position (reverse order to apply from end to start)
        let mut sorted_edits = edits;
        sorted_edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });

        for edit in sorted_edits {
            self.apply_text_edit(edit);
        }

        self.version += 1;
        self.content.clone()
    }

    fn apply_text_edit(&mut self, edit: TextEdit) {
        if let (Some(start_offset), Some(end_offset)) = (
            self.get_position_offset(edit.range.start),
            self.get_position_offset(edit.range.end),
        ) {
            let before = &self.content[..start_offset.min(self.content.len())];
            let after = &self.content[end_offset.min(self.content.len())..];
            self.content = format!("{}{}{}", before, edit.new_text, after);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_doc(content: &str) -> Document {
        Document::new(
            Url::parse("file:///test.txt").unwrap(),
            content.to_string(),
            "text".to_string(),
            1,
        )
    }

    #[test]
    fn test_get_position_offset_ascii() {
        let doc = create_test_doc("Hello\nWorld");
        
        // First line
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 0 }), Some(0));
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 5 }), Some(5));
        
        // Second line
        assert_eq!(doc.get_position_offset(Position { line: 1, character: 0 }), Some(6));
        assert_eq!(doc.get_position_offset(Position { line: 1, character: 5 }), Some(11));
    }

    #[test]
    fn test_get_position_offset_japanese() {
        let doc = create_test_doc("こんにちは\n世界");
        
        // Japanese characters: each char is 3 bytes but 1 UTF-16 code unit
        // "こ" = 3 bytes, 1 UTF-16 unit
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 0 }), Some(0));
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 1 }), Some(3));
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 5 }), Some(15));
        
        // Second line
        assert_eq!(doc.get_position_offset(Position { line: 1, character: 0 }), Some(16));
        assert_eq!(doc.get_position_offset(Position { line: 1, character: 2 }), Some(22));
    }

    #[test]
    fn test_get_position_offset_emoji() {
        let doc = create_test_doc("Hello😀World");
        
        // Emoji "😀" is 4 bytes and 2 UTF-16 code units
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 0 }), Some(0));
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 5 }), Some(5));
        assert_eq!(doc.get_position_offset(Position { line: 0, character: 7 }), Some(9)); // After emoji
    }

    #[test]
    fn test_offset_to_position_ascii() {
        let doc = create_test_doc("Hello\nWorld");
        
        assert_eq!(doc.offset_to_position(0), Position { line: 0, character: 0 });
        assert_eq!(doc.offset_to_position(5), Position { line: 0, character: 5 });
        assert_eq!(doc.offset_to_position(6), Position { line: 1, character: 0 });
        assert_eq!(doc.offset_to_position(11), Position { line: 1, character: 5 });
    }

    #[test]
    fn test_offset_to_position_japanese() {
        let doc = create_test_doc("こんにちは\n世界");
        
        assert_eq!(doc.offset_to_position(0), Position { line: 0, character: 0 });
        assert_eq!(doc.offset_to_position(3), Position { line: 0, character: 1 });
        assert_eq!(doc.offset_to_position(15), Position { line: 0, character: 5 });
        assert_eq!(doc.offset_to_position(16), Position { line: 1, character: 0 });
    }

    #[test]
    fn test_offset_to_position_emoji() {
        let doc = create_test_doc("Hello😀World");
        
        assert_eq!(doc.offset_to_position(0), Position { line: 0, character: 0 });
        assert_eq!(doc.offset_to_position(5), Position { line: 0, character: 5 });
        assert_eq!(doc.offset_to_position(9), Position { line: 0, character: 7 }); // After emoji
    }

    #[test]
    fn test_apply_text_edit() {
        let mut doc = create_test_doc("Hello World");
        
        let edit = TextEdit {
            range: Range {
                start: Position { line: 0, character: 6 },
                end: Position { line: 0, character: 11 },
            },
            new_text: "Rust".to_string(),
        };
        
        doc.apply_text_edit(edit);
        assert_eq!(doc.content, "Hello Rust");
    }

    #[test]
    fn test_get_word_at_position() {
        let doc = create_test_doc("hello_world test");
        
        assert_eq!(
            doc.get_word_at_position(Position { line: 0, character: 0 }),
            Some("hello_world".to_string())
        );
        assert_eq!(
            doc.get_word_at_position(Position { line: 0, character: 12 }),
            Some("test".to_string())
        );
    }
}
