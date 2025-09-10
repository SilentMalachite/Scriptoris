use anyhow::Result;
use lsp_types::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc;
use serde_json::Value;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct LspServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
    diagnostics: Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
}

impl LspServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        self.documents
            .write()
            .await
            .insert(params.uri.clone(), params.text);

        // Perform validation and send diagnostics
        let diagnostics = self.validate_document(&params.uri).await;
        self.diagnostics
            .write()
            .await
            .insert(params.uri.clone(), diagnostics.clone());

        self.client
            .publish_diagnostics(params.uri, diagnostics, Some(params.version))
            .await;
    }

    async fn validate_document(&self, uri: &Url) -> Vec<Diagnostic> {
        // This is where you would implement actual validation logic
        // For now, we'll return an empty vector
        vec![]
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LspServer {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: None,
                declaration_provider: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                definition_provider: Some(OneOf::Left(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                document_on_type_formatting_provider: None,
                rename_provider: Some(OneOf::Left(true)),
                document_link_provider: None,
                color_provider: None,
                folding_range_provider: None,
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![],
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                selection_range_provider: None,
                linked_editing_range_provider: None,
                call_hierarchy_provider: None,
                semantic_tokens_provider: None,
                moniker_provider: None,

                inline_value_provider: None,
                inlay_hint_provider: None,
                diagnostic_provider: None,
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                experimental: None,
            },
            server_info: Some(ServerInfo {
                name: "scriptoris-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            language_id: params.text_document.language_id,
            version: params.text_document.version,
            text: params.text_document.text,
        })
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        
        // For simplicity, we assume full document sync
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.write().await.insert(uri.clone(), change.text);
            
            let diagnostics = self.validate_document(&uri).await;
            self.diagnostics.write().await.insert(uri.clone(), diagnostics.clone());
            
            self.client
                .publish_diagnostics(uri, diagnostics, Some(version))
                .await;
        }
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        // Handle save event if needed
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
        self.diagnostics.write().await.remove(&params.text_document.uri);
    }

    async fn completion(&self, _: CompletionParams) -> jsonrpc::Result<Option<CompletionResponse>> {
        // Return sample completions
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem {
                label: "example".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Example completion".to_string()),
                documentation: Some(Documentation::String("This is an example completion item".to_string())),
                ..Default::default()
            },
        ])))
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        // Return sample hover information
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("Hover at line {}, character {}", position.line, position.character),
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        _: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }

    async fn references(&self, _: ReferenceParams) -> jsonrpc::Result<Option<Vec<Location>>> {
        Ok(None)
    }

    async fn document_highlight(
        &self,
        _: DocumentHighlightParams,
    ) -> jsonrpc::Result<Option<Vec<DocumentHighlight>>> {
        Ok(None)
    }

    async fn document_symbol(
        &self,
        _: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        Ok(None)
    }

    async fn code_action(&self, _: CodeActionParams) -> jsonrpc::Result<Option<CodeActionResponse>> {
        Ok(None)
    }

    async fn code_lens(&self, _: CodeLensParams) -> jsonrpc::Result<Option<Vec<CodeLens>>> {
        Ok(None)
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> jsonrpc::Result<CodeLens> {
        Ok(params)
    }

    async fn formatting(&self, _: DocumentFormattingParams) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        Ok(None)
    }

    async fn rename(&self, _: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        Ok(None)
    }

    async fn prepare_rename(
        &self,
        _: TextDocumentPositionParams,
    ) -> jsonrpc::Result<Option<PrepareRenameResponse>> {
        Ok(None)
    }

    async fn signature_help(&self, _: SignatureHelpParams) -> jsonrpc::Result<Option<SignatureHelp>> {
        Ok(None)
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> jsonrpc::Result<Option<Value>> {
        Ok(None)
    }
}

#[allow(dead_code)]
pub async fn run_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| LspServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}