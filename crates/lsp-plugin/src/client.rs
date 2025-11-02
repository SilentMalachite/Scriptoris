use anyhow::Result;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::timeout;

#[derive(Debug)]
pub struct LspClient {
    process: Arc<Mutex<TokioChild>>,
    request_id: Arc<AtomicI32>,
    pending_requests: Arc<RwLock<HashMap<i32, mpsc::Sender<Value>>>>,
    shutdown: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: i32,
    method: String,
    params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl LspClient {
    pub async fn new(command: String, args: Vec<String>) -> Result<Self> {
        let process = TokioCommand::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let client = Self {
            process: Arc::new(Mutex::new(process)),
            request_id: Arc::new(AtomicI32::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        // Start message handler
        client.start_message_handler().await;

        Ok(client)
    }

    async fn start_message_handler(&self) {
        let process = self.process.clone();
        let pending_requests = self.pending_requests.clone();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            let mut process = process.lock().await;
            if let Some(stdout) = process.stdout.take() {
                let mut reader = AsyncBufReader::new(stdout);
                let mut buffer = String::new();

                loop {
                    // Check shutdown flag
                    if shutdown.load(Ordering::Relaxed) {
                        log::info!("LSP message handler shutting down");
                        break;
                    }

                    buffer.clear();
                    // Read headers with timeout
                    let read_result = timeout(Duration::from_secs(30), async {
                        loop {
                            let mut line = String::new();
                            if reader.read_line(&mut line).await.is_err() {
                                return None;
                            }
                            if line == "\r\n" || line == "\n" {
                                break;
                            }
                            if let Some(stripped) = line.strip_prefix("Content-Length: ") {
                                if let Ok(len) = stripped.trim().parse::<usize>() {
                                    // Read content
                                    let mut content = vec![0u8; len];
                                    if reader.read_exact(&mut content).await.is_ok() {
                                        if let Ok(text) = String::from_utf8(content) {
                                            return Some(text);
                                        }
                                    }
                                }
                            }
                        }
                        None
                    })
                    .await;

                    match read_result {
                        Ok(Some(text)) => {
                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                // Handle response or notification
                                if value.get("id").is_some() {
                                    // Response
                                    if let Ok(response) =
                                        serde_json::from_value::<JsonRpcResponse>(value.clone())
                                    {
                                        if let Some(sender) =
                                            pending_requests.write().await.remove(&response.id)
                                        {
                                            let _ = sender.send(value).await;
                                        }
                                    }
                                } else {
                                    // Notification - log but don't handle
                                    if let Ok(notif) =
                                        serde_json::from_value::<JsonRpcNotification>(value)
                                    {
                                        log::debug!("Received LSP notification: {}", notif.method);
                                    }
                                }
                            }
                        }
                        Ok(None) | Err(_) => {
                            // Timeout or error - exit loop
                            log::warn!("LSP message handler timeout or error");
                            break;
                        }
                    }
                }

                // Clean up pending requests on exit
                pending_requests.write().await.clear();
            }
        });
    }

    async fn send_request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: serde_json::to_value(params)?,
        };

        let (tx, mut rx) = mpsc::channel(1);
        self.pending_requests.write().await.insert(id, tx);

        self.send_message(&request).await?;

        // Wait for response with timeout
        match timeout(Duration::from_secs(30), rx.recv()).await {
            Ok(Some(response)) => {
                if let Some(result) = response.get("result") {
                    Ok(serde_json::from_value(result.clone())?)
                } else if let Some(error) = response.get("error") {
                    Err(anyhow::anyhow!("LSP error: {:?}", error))
                } else {
                    Err(anyhow::anyhow!("Invalid LSP response"))
                }
            }
            Ok(None) => {
                // Clean up pending request
                self.pending_requests.write().await.remove(&id);
                Err(anyhow::anyhow!("No response received"))
            }
            Err(_) => {
                // Timeout - clean up pending request
                self.pending_requests.write().await.remove(&id);
                Err(anyhow::anyhow!("Request timeout"))
            }
        }
    }

    async fn send_notification<P: Serialize>(&self, method: &str, params: P) -> Result<()> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: serde_json::to_value(params)?,
        };

        self.send_message(&notification).await
    }

    async fn send_message<T: Serialize>(&self, message: &T) -> Result<()> {
        let json = serde_json::to_string(message)?;
        let content = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);

        let mut process = self.process.lock().await;
        if let Some(stdin) = process.stdin.as_mut() {
            stdin.write_all(content.as_bytes()).await?;
            stdin.flush().await?;
        }

        Ok(())
    }

    // LSP Methods
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.send_request("initialize", params).await
    }

    pub async fn initialized(&self) -> Result<()> {
        self.send_notification("initialized", InitializedParams {})
            .await
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Set shutdown flag first
        self.shutdown.store(true, Ordering::Relaxed);

        // Send shutdown request
        let result = self.send_request::<(), ()>("shutdown", ()).await;

        // Send exit notification
        let _ = self.send_notification("exit", ()).await;

        // Clean up pending requests
        self.pending_requests.write().await.clear();

        // Kill the process if still running
        let mut process = self.process.lock().await;
        let _ = process.kill().await;

        result
    }

    pub async fn exit(&self) -> Result<()> {
        self.send_notification("exit", ()).await
    }

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
        self.send_notification("textDocument/didOpen", params).await
    }

    pub async fn did_change(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        self.send_notification("textDocument/didChange", params)
            .await
    }

    pub async fn did_save(&self, params: DidSaveTextDocumentParams) -> Result<()> {
        self.send_notification("textDocument/didSave", params).await
    }

    pub async fn did_close(&self, params: DidCloseTextDocumentParams) -> Result<()> {
        self.send_notification("textDocument/didClose", params)
            .await
    }

    pub async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.send_request("textDocument/completion", params).await
    }

    pub async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.send_request("textDocument/hover", params).await
    }

    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.send_request("textDocument/definition", params).await
    }

    pub async fn goto_type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.send_request("textDocument/typeDefinition", params)
            .await
    }

    pub async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.send_request("textDocument/implementation", params)
            .await
    }

    pub async fn find_references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.send_request("textDocument/references", params).await
    }

    pub async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.send_request("textDocument/documentHighlight", params)
            .await
    }

    pub async fn document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.send_request("textDocument/documentSymbol", params)
            .await
    }

    pub async fn workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        self.send_request("workspace/symbol", params).await
    }

    pub async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        self.send_request("textDocument/codeAction", params).await
    }

    pub async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        self.send_request("textDocument/codeLens", params).await
    }

    pub async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/formatting", params).await
    }

    pub async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.send_request("textDocument/rangeFormatting", params)
            .await
    }

    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.send_request("textDocument/rename", params).await
    }

    pub async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        self.send_request("textDocument/prepareRename", params)
            .await
    }

    pub async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        self.send_request("textDocument/signatureHelp", params)
            .await
    }

    pub async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        self.send_request("workspace/executeCommand", params).await
    }
}

// Implement Drop to ensure cleanup
impl Drop for LspClient {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
