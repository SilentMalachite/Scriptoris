use anyhow::Result;
use crate::app::{BufferManager, WindowManager};
use std::path::PathBuf;

use crate::editor::Editor;
use crate::file_manager::FileManager;

pub struct CommandProcessor;

impl CommandProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_command(
        &self, 
        command: &str, 
        editor: &mut Editor, 
        file_manager: &mut FileManager,
        buffer_manager: &mut BufferManager,
        window_manager: &mut WindowManager,
        should_quit: &mut bool
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
                    buffer_manager.open_file(path)?;
                    let buffer = buffer_manager.get_current_mut();
                    *editor = buffer.content.clone();
                    Ok("File opened in new buffer".to_string())
                } else {
                    Err(anyhow::anyhow!("E471: Argument required"))
                }
            }
            "split" | "sp" => {
                // :split - horizontal split
                let buffer_id = if parts.len() > 1 {
                    let path = PathBuf::from(parts[1]);
                    buffer_manager.open_file(path)?
                } else {
                    buffer_manager.get_current().id
                };
                window_manager.split_horizontal(buffer_id);
                Ok("Window split horizontally".to_string())
            }
            "vsplit" | "vsp" => {
                // :vsplit - vertical split
                let buffer_id = if parts.len() > 1 {
                    let path = PathBuf::from(parts[1]);
                    buffer_manager.open_file(path)?
                } else {
                    buffer_manager.get_current().id
                };
                window_manager.split_vertical(buffer_id);
                Ok("Window split vertically".to_string())
            }
            "bnext" | "bn" => {
                // :bnext - next buffer
                buffer_manager.next_buffer();
                let buffer = buffer_manager.get_current_mut();
                *editor = buffer.content.clone();
                Ok(format!("Buffer {}", buffer.id))
            }
            "bprev" | "bp" => {
                // :bprev - previous buffer
                buffer_manager.prev_buffer();
                let buffer = buffer_manager.get_current_mut();
                *editor = buffer.content.clone();
                Ok(format!("Buffer {}", buffer.id))
            }
            "buffers" | "ls" => {
                // :buffers - list buffers
                let buffers = buffer_manager.list_buffers();
                let mut output = String::new();
                for (id, path, modified, current) in buffers {
                    let marker = if current { "%" } else { " " };
                    let mod_marker = if modified { "+" } else { " " };
                    let name = path.map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "[No Name]".to_string());
                    output.push_str(&format!("{}{} {} {}
", marker, mod_marker, id, name));
                }
                Ok(output.trim_end().to_string())
            }
            "bdelete" | "bd" => {
                // :bdelete - delete buffer
                if parts.len() > 1 {
                    let id: usize = parts[1].parse()?;
                    buffer_manager.close_buffer(id)?;
                    Ok(format!("Buffer {} deleted", id))
                } else {
                    let id = buffer_manager.get_current().id;
                    buffer_manager.close_buffer(id)?;
                    Ok(format!("Buffer {} deleted", id))
                }
            }
            // Session management commands - TODO: implement with proper session manager access
            "mksession" | "source" | "sessions" | "delsession" => {
                Ok("Session management commands not yet implemented".to_string())
            }
            _ => {
                Err(anyhow::anyhow!("E492: Not an editor command: {}", parts[0]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_command_processor_creation() {
        let cp = CommandProcessor::new();
        // Just ensure it can be created
        assert_eq!(std::mem::size_of_val(&cp), 0); // Zero-sized struct
    }

    #[tokio::test]
    async fn test_quit_commands() {
        let cp = CommandProcessor::new();
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        // Test quit with unmodified editor
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command("q", &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
        assert!(result.is_ok());
        assert!(should_quit);

        // Reset
        should_quit = false;
        editor.insert_char('a'); // Make editor modified

        // Test quit with modified editor
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command("q", &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
        assert!(result.is_ok());
        assert!(!should_quit); // Should not quit due to modifications
        assert!(result.unwrap().contains("No write since last change"));

        // Test force quit
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command("q!", &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
        assert!(result.is_ok());
        assert!(should_quit); // Should force quit
    }

    #[tokio::test]
    async fn test_search_command() {
        let cp = CommandProcessor::new();
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        editor.set_content("Hello World\nTest line".to_string());
        
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command("search World", &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
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
        
        let cp = CommandProcessor::new();
        let mut editor = Editor::new();
        let mut file_manager = FileManager::new();
        let mut should_quit = false;

        // Test edit command
        let cmd = format!("e {}", temp_file.path().display());
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command(&cmd, &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
        assert!(result.is_ok());
        assert_eq!(editor.get_content(), "Initial content\n");

        // Test save command
        editor.insert_char('!');
        let mut buffer_manager = BufferManager::new();
        let mut window_manager = WindowManager::new(0);
        let result = cp.execute_command("w", &mut editor, &mut file_manager, &mut buffer_manager, &mut window_manager, &mut should_quit).await;
        assert!(result.is_ok());
        assert!(!editor.is_modified());
    }
}