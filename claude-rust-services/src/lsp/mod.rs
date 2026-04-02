use async_trait::async_trait;
use claude_rust_errors::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub file_path: String,
    pub line: u32,
    pub col: u32,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file_path: String,
    pub line: u32,
    pub col: u32,
}

#[async_trait]
pub trait LspService: Send + Sync {
    async fn start_server(&self, language: &str) -> AppResult<()>;
    async fn diagnostics(&self, file_path: &str) -> AppResult<Vec<Diagnostic>>;
    async fn hover(&self, file_path: &str, line: u32, col: u32) -> AppResult<Option<HoverInfo>>;
    async fn goto_definition(&self, file_path: &str, line: u32, col: u32) -> AppResult<Option<Location>>;
    async fn references(&self, file_path: &str, line: u32, col: u32) -> AppResult<Vec<Location>>;
    async fn shutdown(&self) -> AppResult<()>;
}

pub struct StubLspService;

impl StubLspService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LspService for StubLspService {
    async fn start_server(&self, language: &str) -> AppResult<()> {
        tracing::warn!(language, "LSP not yet initialized");
        Ok(())
    }

    async fn diagnostics(&self, _file_path: &str) -> AppResult<Vec<Diagnostic>> {
        tracing::warn!("LSP not yet initialized");
        Ok(Vec::new())
    }

    async fn hover(&self, _file_path: &str, _line: u32, _col: u32) -> AppResult<Option<HoverInfo>> {
        tracing::warn!("LSP not yet initialized");
        Ok(None)
    }

    async fn goto_definition(&self, _file_path: &str, _line: u32, _col: u32) -> AppResult<Option<Location>> {
        tracing::warn!("LSP not yet initialized");
        Ok(None)
    }

    async fn references(&self, _file_path: &str, _line: u32, _col: u32) -> AppResult<Vec<Location>> {
        tracing::warn!("LSP not yet initialized");
        Ok(Vec::new())
    }

    async fn shutdown(&self) -> AppResult<()> {
        tracing::warn!("LSP not yet initialized");
        Ok(())
    }
}
