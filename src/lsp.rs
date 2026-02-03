//! Language Server Protocol client foundation for Quirks
//!
//! Provides basic LSP communication infrastructure.
//! Full implementation will require async runtime and JSON-RPC.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::io::{BufReader, BufWriter, Write, BufRead};

/// LSP server configuration
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    /// Server command (e.g., "rust-analyzer", "pyright")
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// File extensions this server handles
    pub extensions: Vec<String>,
    /// Language ID for the server
    pub language_id: String,
}

impl LspServerConfig {
    pub fn new(command: &str, language_id: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            extensions: Vec::new(),
            language_id: language_id.to_string(),
        }
    }

    pub fn with_args(mut self, args: Vec<&str>) -> Self {
        self.args = args.into_iter().map(String::from).collect();
        self
    }

    pub fn with_extensions(mut self, exts: Vec<&str>) -> Self {
        self.extensions = exts.into_iter().map(String::from).collect();
        self
    }
}

/// Default LSP server configurations
pub fn default_servers() -> HashMap<String, LspServerConfig> {
    let mut servers = HashMap::new();
    
    // Rust
    servers.insert(
        "rust".to_string(),
        LspServerConfig::new("rust-analyzer", "rust")
            .with_extensions(vec!["rs"]),
    );
    
    // Python
    servers.insert(
        "python".to_string(),
        LspServerConfig::new("pyright-langserver", "python")
            .with_args(vec!["--stdio"])
            .with_extensions(vec!["py"]),
    );
    
    // TypeScript/JavaScript
    servers.insert(
        "typescript".to_string(),
        LspServerConfig::new("typescript-language-server", "typescript")
            .with_args(vec!["--stdio"])
            .with_extensions(vec!["ts", "tsx", "js", "jsx"]),
    );
    
    // Go
    servers.insert(
        "go".to_string(),
        LspServerConfig::new("gopls", "go")
            .with_extensions(vec!["go"]),
    );
    
    // C/C++
    servers.insert(
        "c".to_string(),
        LspServerConfig::new("clangd", "c")
            .with_extensions(vec!["c", "h", "cpp", "hpp", "cc", "cxx"]),
    );
    
    servers
}

/// Position in a document (LSP uses 0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// A range in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// A text edit
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// A diagnostic message from the server
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

/// Completion item kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// A completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

/// LSP client state
#[derive(Debug)]
pub struct LspClient {
    config: LspServerConfig,
    process: Option<Child>,
    request_id: i64,
    initialized: bool,
}

impl LspClient {
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            config,
            process: None,
            request_id: 0,
            initialized: false,
        }
    }

    /// Start the LSP server process
    pub fn start(&mut self) -> Result<(), String> {
        let child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start LSP server: {}", e))?;
        
        self.process = Some(child);
        Ok(())
    }

    /// Stop the LSP server
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
        self.initialized = false;
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    /// Get the next request ID
    fn next_id(&mut self) -> i64 {
        self.request_id += 1;
        self.request_id
    }

    /// Format a JSON-RPC request
    fn format_request(&self, method: &str, params: &str) -> String {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"{}\",\"params\":{}}}",
            self.request_id, method, params
        )
    }

    /// Format a JSON-RPC notification (no response expected)
    fn format_notification(&self, method: &str, params: &str) -> String {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"{}\",\"params\":{}}}",
            method, params
        )
    }

    /// Send a message to the server (with Content-Length header)
    fn send_message(&mut self, content: &str) -> Result<(), String> {
        let process = self.process.as_mut()
            .ok_or("LSP server not running")?;
        
        let stdin = process.stdin.as_mut()
            .ok_or("Failed to get stdin")?;
        
        let message = format!(
            "Content-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        
        stdin.write_all(message.as_bytes())
            .map_err(|e| format!("Failed to write to LSP: {}", e))?;
        stdin.flush()
            .map_err(|e| format!("Failed to flush LSP stdin: {}", e))?;
        
        Ok(())
    }

    /// Initialize the server (must be called first)
    pub fn initialize(&mut self, root_uri: &str) -> Result<(), String> {
        let _id = self.next_id();
        
        let params = format!(
            r#"{{"processId":{},"rootUri":"{}","capabilities":{{}}}}"#,
            std::process::id(),
            root_uri
        );
        
        let request = self.format_request("initialize", &params);
        self.send_message(&request)?;
        
        // In a real implementation, we'd wait for the response
        // For now, just mark as initialized
        self.initialized = true;
        
        // Send initialized notification
        let notification = self.format_notification("initialized", "{}");
        self.send_message(&notification)?;
        
        Ok(())
    }

    /// Notify the server that a document was opened
    pub fn did_open(&mut self, uri: &str, language_id: &str, version: i32, text: &str) -> Result<(), String> {
        let params = format!(
            r#"{{"textDocument":{{"uri":"{}","languageId":"{}","version":{},"text":"{}"}}}}"#,
            uri,
            language_id,
            version,
            text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
        );
        
        let notification = self.format_notification("textDocument/didOpen", &params);
        self.send_message(&notification)
    }

    /// Notify the server that a document was changed
    pub fn did_change(&mut self, uri: &str, version: i32, text: &str) -> Result<(), String> {
        let params = format!(
            r#"{{"textDocument":{{"uri":"{}","version":{}}},"contentChanges":[{{"text":"{}"}}]}}"#,
            uri,
            version,
            text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
        );
        
        let notification = self.format_notification("textDocument/didChange", &params);
        self.send_message(&notification)
    }

    /// Notify the server that a document was closed
    pub fn did_close(&mut self, uri: &str) -> Result<(), String> {
        let params = format!(r#"{{"textDocument":{{"uri":"{}"}}}}"#, uri);
        
        let notification = self.format_notification("textDocument/didClose", &params);
        self.send_message(&notification)
    }

    /// Request completions at a position
    pub fn completion(&mut self, uri: &str, line: u32, character: u32) -> Result<(), String> {
        let _id = self.next_id();
        
        let params = format!(
            r#"{{"textDocument":{{"uri":"{}"}},"position":{{"line":{},"character":{}}}}}"#,
            uri, line, character
        );
        
        let request = self.format_request("textDocument/completion", &params);
        self.send_message(&request)
    }

    /// Request hover information at a position
    pub fn hover(&mut self, uri: &str, line: u32, character: u32) -> Result<(), String> {
        let _id = self.next_id();
        
        let params = format!(
            r#"{{"textDocument":{{"uri":"{}"}},"position":{{"line":{},"character":{}}}}}"#,
            uri, line, character
        );
        
        let request = self.format_request("textDocument/hover", &params);
        self.send_message(&request)
    }

    /// Request go-to-definition
    pub fn goto_definition(&mut self, uri: &str, line: u32, character: u32) -> Result<(), String> {
        let _id = self.next_id();
        
        let params = format!(
            r#"{{"textDocument":{{"uri":"{}"}},"position":{{"line":{},"character":{}}}}}"#,
            uri, line, character
        );
        
        let request = self.format_request("textDocument/definition", &params);
        self.send_message(&request)
    }

    /// Shutdown the server gracefully
    pub fn shutdown(&mut self) -> Result<(), String> {
        let _id = self.next_id();
        let request = self.format_request("shutdown", "null");
        self.send_message(&request)?;
        
        let notification = self.format_notification("exit", "null");
        self.send_message(&notification)?;
        
        self.stop();
        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Find the appropriate LSP server for a file
pub fn server_for_file(path: &str, servers: &HashMap<String, LspServerConfig>) -> Option<&LspServerConfig> {
    let path = PathBuf::from(path);
    let ext = path.extension()?.to_str()?;
    
    servers.values().find(|config| {
        config.extensions.iter().any(|e| e == ext)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_default_servers() {
        let servers = default_servers();
        assert!(servers.contains_key("rust"));
        assert!(servers.contains_key("python"));
        assert!(servers.contains_key("typescript"));
    }

    #[test]
    fn test_server_for_file() {
        let servers = default_servers();
        
        let rust_server = server_for_file("main.rs", &servers);
        assert!(rust_server.is_some());
        assert_eq!(rust_server.unwrap().language_id, "rust");
        
        let python_server = server_for_file("script.py", &servers);
        assert!(python_server.is_some());
        assert_eq!(python_server.unwrap().language_id, "python");
    }

    #[test]
    fn test_lsp_client_creation() {
        let config = LspServerConfig::new("test-server", "test");
        let client = LspClient::new(config);
        assert!(!client.is_running());
    }
}
