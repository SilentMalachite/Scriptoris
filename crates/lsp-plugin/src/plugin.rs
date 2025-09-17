use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lsp_types::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::LspPlugin as LspCore;

pub struct ScriptorisLspPlugin {
    lsp: Arc<LspCore>,
    enabled: Arc<RwLock<bool>>,
    current_file: Arc<RwLock<Option<PathBuf>>>,
    completions: Arc<RwLock<Vec<CompletionItem>>>,
    diagnostics_shown: Arc<RwLock<bool>>,
}

impl ScriptorisLspPlugin {
    pub fn new() -> Self {
        Self {
            lsp: Arc::new(LspCore::new()),
            enabled: Arc::new(RwLock::new(true)),
            current_file: Arc::new(RwLock::new(None)),
            completions: Arc::new(RwLock::new(Vec::new())),
            diagnostics_shown: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Load config if exists
        let project_dirs = directories::ProjectDirs::from("com", "scriptoris", "scriptoris")
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let config_path = project_dirs.config_dir().join("lsp.json");

        if config_path.exists() {
            self.lsp.load_config(config_path).await?;
        }

        // Start language servers based on config
        let config = self.lsp.config.read().await;
        if config.auto_start {
            for (name, _) in &config.servers {
                // Try to start each configured server
                let _ = self.lsp.start_server(name).await;
            }
        }

        Ok(())
    }

    pub async fn open_file(&self, path: PathBuf, content: String) -> Result<()> {
        *self.current_file.write().await = Some(path.clone());

        // Detect language from file extension
        let language_id = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust",
            Some("ts") | Some("tsx") => "typescript",
            Some("js") | Some("jsx") => "javascript",
            Some("py") => "python",
            Some("go") => "go",
            Some("c") => "c",
            Some("cpp") | Some("cc") | Some("cxx") => "cpp",
            Some("java") => "java",
            Some("md") => "markdown",
            Some("json") => "json",
            Some("yaml") | Some("yml") => "yaml",
            Some("toml") => "toml",
            _ => "plaintext",
        }
        .to_string();

        self.lsp.open_document(path, content, language_id).await?;
        Ok(())
    }

    pub async fn update_file(&self, content: String, version: i32) -> Result<()> {
        if let Some(path) = self.current_file.read().await.as_ref() {
            self.lsp
                .update_document(path.clone(), content, version)
                .await?;
        }
        Ok(())
    }

    pub async fn get_completions_at_cursor(
        &self,
        line: u32,
        character: u32,
    ) -> Result<Vec<CompletionItem>> {
        if let Some(path) = self.current_file.read().await.as_ref() {
            let completions = self
                .lsp
                .get_completions(path.clone(), line, character)
                .await?;
            *self.completions.write().await = completions.clone();
            Ok(completions)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_hover_at_cursor(&self, line: u32, character: u32) -> Result<Option<String>> {
        if let Some(path) = self.current_file.read().await.as_ref() {
            if let Some(hover) = self.lsp.get_hover(path.clone(), line, character).await? {
                let content = match hover.contents {
                    HoverContents::Scalar(MarkedString::String(s)) => s,
                    HoverContents::Scalar(MarkedString::LanguageString(ls)) => {
                        format!("```{}\n{}\n```", ls.language, ls.value)
                    }
                    HoverContents::Array(items) => items
                        .into_iter()
                        .map(|item| match item {
                            MarkedString::String(s) => s,
                            MarkedString::LanguageString(ls) => {
                                format!("```{}\n{}\n```", ls.language, ls.value)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n"),
                    HoverContents::Markup(markup) => markup.value,
                };
                Ok(Some(content))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn goto_definition_at_cursor(
        &self,
        line: u32,
        character: u32,
    ) -> Result<Option<Location>> {
        if let Some(path) = self.current_file.read().await.as_ref() {
            if let Some(response) = self
                .lsp
                .goto_definition(path.clone(), line, character)
                .await?
            {
                match response {
                    GotoDefinitionResponse::Scalar(location) => Ok(Some(location)),
                    GotoDefinitionResponse::Array(locations) => Ok(locations.into_iter().next()),
                    GotoDefinitionResponse::Link(links) => {
                        Ok(links.into_iter().next().map(|link| Location {
                            uri: link.target_uri,
                            range: link.target_selection_range,
                        }))
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_diagnostics(&self) -> Result<Vec<Diagnostic>> {
        if let Some(path) = self.current_file.read().await.as_ref() {
            let uri =
                Url::from_file_path(path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;
            Ok(self.lsp.get_diagnostics(&uri).await)
        } else {
            Ok(vec![])
        }
    }

    pub async fn format_current_buffer(&self) -> Result<Option<String>> {
        // TODO: Implement formatting
        Ok(None)
    }
}

// Implement the Scriptoris Plugin trait
#[async_trait]
impl scriptoris::app::Plugin for ScriptorisLspPlugin {
    fn name(&self) -> &str {
        "LSP Support"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn on_load(&mut self, _app: &mut scriptoris::app::App) -> Result<()> {
        // Initialize LSP on load
        let plugin = self.clone();
        tokio::spawn(async move {
            let _ = plugin.initialize().await;
        });
        Ok(())
    }

    fn on_key(&mut self, app: &mut scriptoris::app::App, key: KeyEvent) -> Result<bool> {
        // Handle LSP-specific key bindings
        match key.code {
            // Ctrl+Space for completion
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let (line, col) = app.get_current_editor().cursor_position();
                let plugin = self.clone();
                tokio::spawn(async move {
                    let _ = plugin
                        .get_completions_at_cursor(line as u32, col as u32)
                        .await;
                });
                Ok(true)
            }
            // Ctrl+K for hover
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let (line, col) = app.get_current_editor().cursor_position();
                let plugin = self.clone();
                tokio::spawn(async move {
                    if let Ok(Some(hover)) =
                        plugin.get_hover_at_cursor(line as u32, col as u32).await
                    {
                        // TODO: Display hover info in UI
                        println!("Hover: {}", hover);
                    }
                });
                Ok(true)
            }
            // Ctrl+] for goto definition
            KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let (line, col) = app.get_current_editor().cursor_position();
                let plugin = self.clone();
                tokio::spawn(async move {
                    if let Ok(Some(location)) = plugin
                        .goto_definition_at_cursor(line as u32, col as u32)
                        .await
                    {
                        // TODO: Navigate to location
                        println!("Go to: {:?}", location);
                    }
                });
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn on_command(
        &mut self,
        _app: &mut scriptoris::app::App,
        command: &str,
    ) -> Result<Option<String>> {
        // Handle LSP-specific commands
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "lsp" => {
                if parts.len() < 2 {
                    return Ok(Some("Usage: :lsp <start|stop|status|restart>".to_string()));
                }
                match parts[1] {
                    "start" => {
                        if parts.len() > 2 {
                            let plugin = self.clone();
                            let server = parts[2].to_string();
                            tokio::spawn(async move {
                                let _ = plugin.lsp.start_server(&server).await;
                            });
                            Ok(Some(format!("Starting LSP server: {}", parts[2])))
                        } else {
                            Ok(Some("Usage: :lsp start <server-name>".to_string()))
                        }
                    }
                    "stop" => {
                        if parts.len() > 2 {
                            let plugin = self.clone();
                            let server = parts[2].to_string();
                            tokio::spawn(async move {
                                let _ = plugin.lsp.stop_server(&server).await;
                            });
                            Ok(Some(format!("Stopping LSP server: {}", parts[2])))
                        } else {
                            Ok(Some("Usage: :lsp stop <server-name>".to_string()))
                        }
                    }
                    "status" => Ok(Some("LSP Status: Active".to_string())),
                    "restart" => Ok(Some("Restarting LSP servers...".to_string())),
                    _ => Ok(Some("Unknown LSP command".to_string())),
                }
            }
            "format" => {
                let plugin = self.clone();
                tokio::spawn(async move {
                    let _ = plugin.format_current_buffer().await;
                });
                Ok(Some("Formatting document...".to_string()))
            }
            _ => Ok(None),
        }
    }

    fn on_save(&mut self, app: &mut scriptoris::app::App, _path: &std::path::Path) -> Result<()> {
        // Update LSP when file is saved
        let content = app.get_current_editor().get_content();
        let plugin = self.clone();
        tokio::spawn(async move {
            let _ = plugin.update_file(content, 1).await;
        });
        Ok(())
    }
}

impl Clone for ScriptorisLspPlugin {
    fn clone(&self) -> Self {
        Self {
            lsp: self.lsp.clone(),
            enabled: self.enabled.clone(),
            current_file: self.current_file.clone(),
            completions: self.completions.clone(),
            diagnostics_shown: self.diagnostics_shown.clone(),
        }
    }
}
