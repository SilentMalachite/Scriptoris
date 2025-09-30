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

    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }

    pub fn has_file(&self) -> bool {
        self.current_path.is_some()
    }

    pub async fn open_file(&mut self, path: PathBuf) -> Result<String> {
        // Validate file path
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "ファイルが見つかりません: {}",
                path.display()
            ));
        }

        if !path.is_file() {
            return Err(anyhow::anyhow!(
                "指定されたパスはファイルではありません: {}",
                path.display()
            ));
        }

        // Check file size (prevent loading extremely large files)
        match fs::metadata(&path).await {
            Ok(metadata) => {
                self.is_readonly = metadata.permissions().readonly();

                // Warn about large files (>10MB)
                const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB
                if metadata.len() > LARGE_FILE_THRESHOLD {
                    log::warn!(
                        "Large file detected ({} bytes): {}",
                        metadata.len(),
                        path.display()
                    );
                }
            }
            Err(e) => {
                log::warn!("Failed to get file metadata: {}", e);
                self.is_readonly = false;
            }
        }

        // Read file with proper error handling
        match fs::read_to_string(&path).await {
            Ok(content) => {
                // Check if content is valid UTF-8
                if content.contains('\0') {
                    return Err(anyhow::anyhow!(
                        "ファイルがバイナリ形式の可能性があります: {}",
                        path.display()
                    ));
                }

                self.current_path = Some(path.clone());
                log::info!("Successfully opened file: {}", path.display());
                Ok(content)
            }
            Err(e) => {
                let error_msg = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        format!("ファイルへのアクセス権限がありません: {}", path.display())
                    }
                    std::io::ErrorKind::NotFound => {
                        format!("ファイルが見つかりません: {}", path.display())
                    }
                    std::io::ErrorKind::InvalidData => {
                        format!("ファイルのエンコーディングが無効です (UTF-8ではありません): {}", path.display())
                    }
                    _ => {
                        format!("ファイル読み込みエラー: {} - {}", path.display(), e)
                    }
                };
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    pub async fn save_file(&self, editor: &mut Editor) -> Result<String> {
        if let Some(ref path) = self.current_path {
            // Check if file is readonly
            if self.is_readonly {
                return Err(anyhow::anyhow!(
                    "ファイルが読み取り専用です: {}",
                    path.display()
                ));
            }

            let content = editor.get_content();

            // Check content size (prevent writing extremely large files)
            const LARGE_CONTENT_THRESHOLD: usize = 50 * 1024 * 1024; // 50MB
            if content.len() > LARGE_CONTENT_THRESHOLD {
                return Err(anyhow::anyhow!(
                    "コンテンツが大きすぎます ({} バイト)。大きなファイルの保存は制限されています。",
                    content.len()
                ));
            }

            // Create backup if file exists and is not empty
            if path.exists() {
                if let Ok(metadata) = fs::metadata(path).await {
                    if metadata.len() > 0 {
                        let backup_path = path.with_extension("bak");
                        if let Err(e) = fs::copy(path, &backup_path).await {
                            log::warn!("Failed to create backup: {}", e);
                        } else {
                            log::info!("Created backup: {}", backup_path.display());
                        }
                    }
                }
            }

            // Attempt to save with retry logic
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 3;

            loop {
                match fs::write(path, content.as_bytes()).await {
                    Ok(_) => {
                        editor.mark_saved();
                        log::info!("Successfully saved file: {}", path.display());
                        return Ok(format!("{} 行を書き込みました", editor.line_count()));
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts >= MAX_ATTEMPTS {
                            let error_msg = match e.kind() {
                                std::io::ErrorKind::PermissionDenied => {
                                    format!("ファイルへの書き込み権限がありません: {}", path.display())
                                }
                                std::io::ErrorKind::WriteZero => {
                                    format!("ディスク容量が不足している可能性があります: {}", path.display())
                                }
                                _ => {
                                    format!("ファイル書き込みエラー: {} - {}", path.display(), e)
                                }
                            };
                            return Err(anyhow::anyhow!(error_msg));
                        }

                        // Wait before retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts as u64)).await;
                        log::warn!("Save attempt {} failed for {}, retrying...", attempts, path.display());
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("ファイルパスが設定されていません"))
        }
    }

    pub async fn save_file_as(&mut self, path: PathBuf, editor: &mut Editor) -> Result<String> {
        // Validate the target path
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                match fs::create_dir_all(parent).await {
                    Ok(_) => {
                        log::info!("Created directory: {}", parent.display());
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "ディレクトリの作成に失敗しました: {} - {}",
                            parent.display(),
                            e
                        ));
                    }
                }
            }
        }

        // Check if file already exists and is not writable
        if path.exists() {
            match fs::metadata(&path).await {
                Ok(metadata) => {
                    if metadata.permissions().readonly() {
                        return Err(anyhow::anyhow!(
                            "ターゲットファイルが読み取り専用です: {}",
                            path.display()
                        ));
                    }
                }
                Err(e) => {
                    log::warn!("Failed to check target file metadata: {}", e);
                }
            }
        }

        let content = editor.get_content();

        // Check content size
        const LARGE_CONTENT_THRESHOLD: usize = 50 * 1024 * 1024; // 50MB
        if content.len() > LARGE_CONTENT_THRESHOLD {
            return Err(anyhow::anyhow!(
                "コンテンツが大きすぎます ({} バイト)。大きなファイルの保存は制限されています。",
                content.len()
            ));
        }

        // Save the file
        match fs::write(&path, content.as_bytes()).await {
            Ok(_) => {
                self.current_path = Some(path.clone());
                self.is_readonly = false;
                editor.mark_saved();
                log::info!("Successfully saved file as: {}", path.display());
                Ok(format!("{} 行を '{}' に書き込みました", editor.line_count(), path.display()))
            }
            Err(e) => {
                let error_msg = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        format!("ファイルへの書き込み権限がありません: {}", path.display())
                    }
                    std::io::ErrorKind::WriteZero => {
                        format!("ディスク容量が不足している可能性があります: {}", path.display())
                    }
                    _ => {
                        format!("ファイル書き込みエラー: {} - {}", path.display(), e)
                    }
                };
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

        let result = fm
            .save_file_as(temp_file.path().to_path_buf(), &mut editor)
            .await;
        assert!(result.is_ok());
        assert!(fm.has_file());
        assert!(!editor.is_modified());
    }
}
