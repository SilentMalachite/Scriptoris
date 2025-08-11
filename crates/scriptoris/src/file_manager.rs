use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

use crate::editor::Editor;

pub struct FileManager {
    pub current_path: Option<PathBuf>,
    pub is_readonly: bool,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            current_path: None,
            is_readonly: false,
        }
    }

    pub fn get_current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    pub fn set_current_file(&mut self, path: PathBuf) {
        self.current_path = Some(path);
    }

    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }
    
    pub fn has_file(&self) -> bool {
        self.current_path.is_some()
    }

    pub async fn open_file(&mut self, path: PathBuf) -> Result<String> {
        // Check if file is readonly
        if let Ok(metadata) = std::fs::metadata(&path) {
            self.is_readonly = metadata.permissions().readonly();
        } else {
            self.is_readonly = false;
        }
        
        let content = fs::read_to_string(&path).await?;
        self.current_path = Some(path);
        Ok(content)
    }

    pub async fn save_file(&self, editor: &mut Editor) -> Result<String> {
        if let Some(ref path) = self.current_path {
            let content = editor.get_content();
            fs::write(path, content).await?;
            editor.mark_saved();
            Ok(format!("Wrote {} lines", editor.line_count()))
        } else {
            Err(anyhow::anyhow!("No file path set"))
        }
    }

    pub async fn save_file_as(&mut self, path: PathBuf, editor: &mut Editor) -> Result<String> {
        let content = editor.get_content();
        fs::write(&path, content).await?;
        self.current_path = Some(path);
        editor.mark_saved();
        Ok(format!("Wrote {} lines", editor.line_count()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_file_manager_creation() {
        let fm = FileManager::new();
        assert!(!fm.has_file());
        assert!(fm.get_current_path().is_none());
    }

    #[tokio::test]
    async fn test_open_and_save_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello World\nTest content").unwrap();
        
        let mut fm = FileManager::new();
        let mut editor = Editor::new();
        
        // Test opening file
        let result = fm.open_file(temp_file.path().to_path_buf()).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        editor.set_content(content);
        assert!(fm.has_file());
        
        // Test saving file
        editor.insert_char('!');
        let result = fm.save_file(&mut editor).await;
        assert!(result.is_ok());
        assert!(!editor.is_modified());
    }

    #[tokio::test]
    async fn test_save_file_as() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut fm = FileManager::new();
        let mut editor = Editor::new();
        
        editor.insert_char('T');
        editor.insert_char('e');
        editor.insert_char('s');
        editor.insert_char('t');
        
        let result = fm.save_file_as(temp_file.path().to_path_buf(), &mut editor).await;
        assert!(result.is_ok());
        assert!(fm.has_file());
        assert!(!editor.is_modified());
    }
}