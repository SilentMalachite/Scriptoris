use anyhow::Result;
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
            let server_names: Vec<String> = config.servers.keys().cloned().collect();
            drop(config);

            let mut errors = Vec::new();
            for name in server_names {
                if let Err(e) = self.lsp.start_server(&name).await {
                    log::error!("Failed to start LSP server {}: {}", name, e);
                    errors.push(format!("{}: {}", name, e));
                }
            }

            if !errors.is_empty() {
                return Err(anyhow::anyhow!(
                    "LSPサーバーの起動に失敗しました: {}",
                    errors.join("; ")
                ));
            }
        }

        Ok(())
    }

    pub async fn open_file(&self, path: PathBuf, content: String) -> Result<()> {
        *self.current_file.write().await = Some(path.clone());

        // Detect language from file extension
        let language_id = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "rust",
            Some("ts" | "tsx") => "typescript",
            Some("js" | "jsx") => "javascript",
            Some("py") => "python",
            Some("go") => "go",
            Some("c") => "c",
            Some("cpp" | "cc" | "cxx") => "cpp",
            Some("java") => "java",
            Some("md") => "markdown",
            Some("json") => "json",
            Some("yaml" | "yml") => "yaml",
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
        let path = {
            let guard = self.current_file.read().await;
            guard.clone()
        };

        let Some(path) = path else {
            return Ok(None);
        };

        let mut options = FormattingOptions::default();
        if options.tab_size == 0 {
            options.tab_size = 4;
        }
        options.insert_spaces = true;
        options.trim_trailing_whitespace.get_or_insert(true);
        options.insert_final_newline.get_or_insert(true);

        self.lsp.format_document(&path, options).await
    }
}

impl Default for ScriptorisLspPlugin {
    fn default() -> Self {
        Self::new()
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
