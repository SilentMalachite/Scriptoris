use anyhow::Result;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

mod capabilities;
mod client;
mod document;
mod plugin;
mod server;

pub use capabilities::get_server_capabilities;
pub use client::LspClient;
pub use document::Document;
pub use plugin::ScriptorisLspPlugin;
pub use server::LspServer;

// LSPプラグイン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub servers: HashMap<String, ServerConfig>,
    pub auto_start: bool,
    pub show_diagnostics_inline: bool,
    pub show_hover_documentation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub filetypes: Vec<String>,
    pub root_markers: Vec<String>,
    pub initialization_options: Option<serde_json::Value>,
    pub settings: Option<serde_json::Value>,
}

impl Default for LspConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();

        // Rust Analyzer
        servers.insert(
            "rust-analyzer".to_string(),
            ServerConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
                filetypes: vec!["rust".to_string(), "rs".to_string()],
                root_markers: vec!["Cargo.toml".to_string()],
                initialization_options: None,
                settings: Some(serde_json::json!({
                    "rust-analyzer": {
                        "cargo": {
                            "allFeatures": true
                        },
                        "procMacro": {
                            "enable": true
                        }
                    }
                })),
            },
        );

        // TypeScript Language Server
        servers.insert(
            "typescript-language-server".to_string(),
            ServerConfig {
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                filetypes: vec![
                    "typescript".to_string(),
                    "javascript".to_string(),
                    "ts".to_string(),
                    "js".to_string(),
                    "tsx".to_string(),
                    "jsx".to_string(),
                ],
                root_markers: vec!["package.json".to_string(), "tsconfig.json".to_string()],
                initialization_options: None,
                settings: None,
            },
        );

        // Python Language Server
        servers.insert(
            "pylsp".to_string(),
            ServerConfig {
                command: "pylsp".to_string(),
                args: vec![],
                filetypes: vec!["python".to_string(), "py".to_string()],
                root_markers: vec![
                    "setup.py".to_string(),
                    "pyproject.toml".to_string(),
                    "requirements.txt".to_string(),
                ],
                initialization_options: None,
                settings: None,
            },
        );

        Self {
            servers,
            auto_start: true,
            show_diagnostics_inline: true,
            show_hover_documentation: true,
        }
    }
}

// LSPプラグイン本体
pub struct LspPlugin {
    config: Arc<RwLock<LspConfig>>,
    clients: Arc<RwLock<HashMap<String, Arc<LspClient>>>>,
    documents: Arc<RwLock<HashMap<PathBuf, Document>>>,
    diagnostics: Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
    current_server: Arc<RwLock<Option<String>>>,
}

impl LspPlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(LspConfig::default())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            current_server: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn load_config(&self, path: PathBuf) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: LspConfig = serde_json::from_str(&content)?;
        *self.config.write().await = config;
        Ok(())
    }

    pub async fn start_server(&self, server_name: &str) -> Result<()> {
        let config = self.config.read().await;
        let server_config = config
            .servers
            .get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server {} not configured", server_name))?;

        let client =
            LspClient::new(server_config.command.clone(), server_config.args.clone()).await?;

        // Initialize
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: None,
            initialization_options: server_config.initialization_options.clone(),
            capabilities: self.get_client_capabilities(),
            trace: Some(TraceValue::Verbose),
            workspace_folders: None,
            client_info: Some(ClientInfo {
                name: "Scriptoris".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            locale: None,
            ..Default::default()
        };

        client.initialize(init_params).await?;
        client.initialized().await?;

        self.clients
            .write()
            .await
            .insert(server_name.to_string(), Arc::new(client));
        *self.current_server.write().await = Some(server_name.to_string());

        Ok(())
    }

    pub async fn stop_server(&self, server_name: &str) -> Result<()> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.remove(server_name) {
            client.shutdown().await?;
        }
        Ok(())
    }

    pub async fn open_document(
        &self,
        path: PathBuf,
        content: String,
        language_id: String,
    ) -> Result<()> {
        let uri = Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        let document = Document::new(uri.clone(), content.clone(), language_id.clone(), 0);
        self.documents.write().await.insert(path.clone(), document);

        // Find appropriate server
        let config = self.config.read().await;
        for (server_name, server_config) in &config.servers {
            if server_config.filetypes.contains(&language_id) {
                if let Some(client) = self.clients.read().await.get(server_name) {
                    let params = DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri,
                            language_id,
                            version: 0,
                            text: content,
                        },
                    };
                    client.did_open(params).await?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn update_document(
        &self,
        path: PathBuf,
        content: String,
        version: i32,
    ) -> Result<()> {
        let uri = Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        if let Some(document) = self.documents.write().await.get_mut(&path) {
            document.update(content.clone(), version);
        }

        // Notify server
        if let Some(server_name) = self.current_server.read().await.as_ref() {
            if let Some(client) = self.clients.read().await.get(server_name) {
                let params = DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier { uri, version },
                    content_changes: vec![TextDocumentContentChangeEvent {
                        range: None,
                        range_length: None,
                        text: content,
                    }],
                };
                client.did_change(params).await?;
            }
        }

        Ok(())
    }

    pub async fn get_completions(
        &self,
        path: PathBuf,
        line: u32,
        character: u32,
    ) -> Result<Vec<CompletionItem>> {
        let uri = Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        if let Some(server_name) = self.current_server.read().await.as_ref() {
            if let Some(client) = self.clients.read().await.get(server_name) {
                let params = CompletionParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position: Position { line, character },
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                    context: None,
                };

                let response = client.completion(params).await?;
                match response {
                    Some(CompletionResponse::Array(items)) => Ok(items),
                    Some(CompletionResponse::List(list)) => Ok(list.items),
                    None => Ok(vec![]),
                }
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_hover(
        &self,
        path: PathBuf,
        line: u32,
        character: u32,
    ) -> Result<Option<Hover>> {
        let uri = Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        if let Some(server_name) = self.current_server.read().await.as_ref() {
            if let Some(client) = self.clients.read().await.get(server_name) {
                let params = HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position: Position { line, character },
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                };

                client.hover(params).await
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn goto_definition(
        &self,
        path: PathBuf,
        line: u32,
        character: u32,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = Url::from_file_path(&path).map_err(|_| anyhow::anyhow!("Invalid file path"))?;

        if let Some(server_name) = self.current_server.read().await.as_ref() {
            if let Some(client) = self.clients.read().await.get(server_name) {
                let params = GotoDefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position: Position { line, character },
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                };

                client.goto_definition(params).await
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
        self.diagnostics
            .read()
            .await
            .get(uri)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn handle_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.write().await.insert(uri, diagnostics);
    }

    fn get_client_capabilities(&self) -> ClientCapabilities {
        ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                apply_edit: Some(true),
                configuration: Some(true),
                workspace_edit: Some(WorkspaceEditClientCapabilities {
                    document_changes: Some(true),
                    resource_operations: Some(vec![
                        ResourceOperationKind::Create,
                        ResourceOperationKind::Rename,
                        ResourceOperationKind::Delete,
                    ]),
                    failure_handling: Some(FailureHandlingKind::Abort),
                    normalizes_line_endings: Some(true),
                    change_annotation_support: None,
                }),
                did_change_configuration: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                did_change_watched_files: None,
                symbol: Some(WorkspaceSymbolClientCapabilities {
                    dynamic_registration: Some(false),
                    symbol_kind: Some(SymbolKindCapability {
                        value_set: Some(vec![
                            SymbolKind::FILE,
                            SymbolKind::MODULE,
                            SymbolKind::NAMESPACE,
                            SymbolKind::PACKAGE,
                            SymbolKind::CLASS,
                            SymbolKind::METHOD,
                            SymbolKind::PROPERTY,
                            SymbolKind::FIELD,
                            SymbolKind::CONSTRUCTOR,
                            SymbolKind::ENUM,
                            SymbolKind::INTERFACE,
                            SymbolKind::FUNCTION,
                            SymbolKind::VARIABLE,
                            SymbolKind::CONSTANT,
                            SymbolKind::STRING,
                            SymbolKind::NUMBER,
                            SymbolKind::BOOLEAN,
                            SymbolKind::ARRAY,
                        ]),
                    }),
                    tag_support: None,
                    resolve_support: None,
                }),
                execute_command: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                workspace_folders: Some(true),
                semantic_tokens: None,
                code_lens: None,
                file_operations: None,
                inline_value: None,
                inlay_hint: None,
                diagnostic: None,
            }),
            text_document: Some(TextDocumentClientCapabilities {
                synchronization: Some(TextDocumentSyncClientCapabilities {
                    dynamic_registration: Some(false),
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    did_save: Some(true),
                }),
                completion: Some(CompletionClientCapabilities {
                    dynamic_registration: Some(false),
                    completion_item: Some(CompletionItemCapability {
                        snippet_support: Some(true),
                        commit_characters_support: Some(true),
                        documentation_format: Some(vec![
                            MarkupKind::Markdown,
                            MarkupKind::PlainText,
                        ]),
                        deprecated_support: Some(true),
                        preselect_support: Some(true),
                        tag_support: None,
                        insert_replace_support: Some(true),
                        resolve_support: None,
                        insert_text_mode_support: None,
                        label_details_support: None,
                    }),
                    completion_item_kind: Some(CompletionItemKindCapability {
                        value_set: Some(vec![
                            CompletionItemKind::TEXT,
                            CompletionItemKind::METHOD,
                            CompletionItemKind::FUNCTION,
                            CompletionItemKind::CONSTRUCTOR,
                            CompletionItemKind::FIELD,
                            CompletionItemKind::VARIABLE,
                            CompletionItemKind::CLASS,
                            CompletionItemKind::INTERFACE,
                            CompletionItemKind::MODULE,
                            CompletionItemKind::PROPERTY,
                            CompletionItemKind::UNIT,
                            CompletionItemKind::VALUE,
                            CompletionItemKind::ENUM,
                            CompletionItemKind::KEYWORD,
                            CompletionItemKind::SNIPPET,
                            CompletionItemKind::COLOR,
                            CompletionItemKind::FILE,
                            CompletionItemKind::REFERENCE,
                        ]),
                    }),
                    context_support: Some(true),
                    insert_text_mode: None,
                    completion_list: None,
                }),
                hover: Some(HoverClientCapabilities {
                    dynamic_registration: Some(false),
                    content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                }),
                signature_help: Some(SignatureHelpClientCapabilities {
                    dynamic_registration: Some(false),
                    signature_information: Some(SignatureInformationSettings {
                        documentation_format: Some(vec![
                            MarkupKind::Markdown,
                            MarkupKind::PlainText,
                        ]),
                        parameter_information: Some(ParameterInformationSettings {
                            label_offset_support: Some(true),
                        }),
                        active_parameter_support: Some(true),
                    }),
                    context_support: Some(true),
                }),
                references: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                document_highlight: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                document_symbol: Some(DocumentSymbolClientCapabilities {
                    dynamic_registration: Some(false),
                    symbol_kind: Some(SymbolKindCapability {
                        value_set: Some(vec![
                            SymbolKind::FILE,
                            SymbolKind::MODULE,
                            SymbolKind::NAMESPACE,
                            SymbolKind::PACKAGE,
                            SymbolKind::CLASS,
                            SymbolKind::METHOD,
                            SymbolKind::PROPERTY,
                            SymbolKind::FIELD,
                            SymbolKind::CONSTRUCTOR,
                            SymbolKind::ENUM,
                            SymbolKind::INTERFACE,
                            SymbolKind::FUNCTION,
                            SymbolKind::VARIABLE,
                            SymbolKind::CONSTANT,
                            SymbolKind::STRING,
                            SymbolKind::NUMBER,
                            SymbolKind::BOOLEAN,
                            SymbolKind::ARRAY,
                        ]),
                    }),
                    hierarchical_document_symbol_support: Some(true),
                    tag_support: None,
                }),
                formatting: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                range_formatting: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                on_type_formatting: Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                declaration: Some(GotoCapability {
                    dynamic_registration: Some(false),
                    link_support: Some(false),
                }),
                definition: Some(GotoCapability {
                    dynamic_registration: Some(false),
                    link_support: Some(false),
                }),
                type_definition: Some(GotoCapability {
                    dynamic_registration: Some(false),
                    link_support: Some(false),
                }),
                implementation: Some(GotoCapability {
                    dynamic_registration: Some(false),
                    link_support: Some(false),
                }),
                code_action: None,
                code_lens: None,
                document_link: None,
                color_provider: None,
                rename: None,
                publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                    related_information: Some(true),
                    tag_support: None,
                    version_support: Some(true),
                    code_description_support: Some(true),
                    data_support: Some(true),
                }),
                folding_range: None,
                selection_range: None,
                linked_editing_range: None,
                call_hierarchy: None,
                semantic_tokens: None,
                moniker: None,
                type_hierarchy: None,
                inline_value: None,
                inlay_hint: None,
                diagnostic: None,
            }),

            window: Some(WindowClientCapabilities {
                work_done_progress: Some(true),
                show_message: Some(ShowMessageRequestClientCapabilities {
                    message_action_item: None,
                }),
                show_document: None,
            }),
            general: None,
            experimental: None,
        }
    }
}
