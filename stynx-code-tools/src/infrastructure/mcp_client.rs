use std::sync::atomic::{AtomicU64, Ordering};

use stynx_code_errors::{AppError, AppResult};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};

use super::mcp_config::McpServerEntry;

pub struct McpClient {
    _process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: AtomicU64,
}

impl McpClient {
    pub async fn start(server_name: &str, entry: &McpServerEntry) -> AppResult<Self> {
        let mut cmd = tokio::process::Command::new(&entry.command);
        cmd.args(&entry.args);
        for (k, v) in &entry.env {
            cmd.env(k, v);
        }
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());

        let mut process = cmd.spawn().map_err(|e| {
            AppError::Tool(format!("MCP '{server_name}' failed to start: {e}"))
        })?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| AppError::Tool("MCP: no stdin".into()))?;
        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .ok_or_else(|| AppError::Tool("MCP: no stdout".into()))?,
        );

        let mut client = Self { _process: process, stdin, stdout, next_id: AtomicU64::new(1) };
        client.initialize().await?;
        Ok(client)
    }

    async fn write_msg(&mut self, msg: &Value) -> AppResult<()> {
        let line = format!("{}\n", serde_json::to_string(msg)?);
        self.stdin.write_all(line.as_bytes()).await
            .map_err(|e| AppError::Tool(format!("MCP write: {e}")))?;
        self.stdin.flush().await
            .map_err(|e| AppError::Tool(format!("MCP flush: {e}")))?;
        Ok(())
    }

    async fn read_msg(&mut self) -> AppResult<Value> {
        let mut line = String::new();
        let timeout = std::time::Duration::from_secs(30);
        let read = tokio::time::timeout(timeout, self.stdout.read_line(&mut line))
            .await
            .map_err(|_| AppError::Tool("MCP server timed out after 30s".into()))?;
        read.map_err(|e| AppError::Tool(format!("MCP read: {e}")))?;
        if line.is_empty() {
            return Err(AppError::Tool("MCP server closed".into()));
        }
        serde_json::from_str(line.trim())
            .map_err(|e| AppError::Tool(format!("MCP parse: {e}")))
    }

    async fn request(&mut self, method: &str, params: Option<Value>) -> AppResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut req = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if let Some(p) = params {
            req["params"] = p;
        }
        self.write_msg(&req).await?;
        let resp = self.read_msg().await?;
        if let Some(err) = resp.get("error") {
            return Err(AppError::Tool(format!("MCP error: {err}")));
        }
        Ok(resp["result"].clone())
    }

    async fn initialize(&mut self) -> AppResult<()> {
        self.request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "stynx-code", "version": "0.4.0"}
            })),
        ).await?;
        self.write_msg(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).await?;
        Ok(())
    }

    pub async fn list_tools(&mut self) -> AppResult<Vec<(String, String, Value)>> {
        let result = self.request("tools/list", None).await?;
        let arr = result["tools"]
            .as_array()
            .ok_or_else(|| AppError::Tool("tools/list: no tools array".into()))?;
        let mut out = Vec::new();
        for t in arr {
            let name = t["name"].as_str().unwrap_or("").to_string();
            let desc = t["description"].as_str().unwrap_or("").to_string();
            let schema = t.get("inputSchema")
                .cloned()
                .unwrap_or_else(|| json!({"type": "object", "properties": {}}));
            if !name.is_empty() {
                out.push((name, desc, schema));
            }
        }
        Ok(out)
    }

    pub async fn call_tool(&mut self, name: &str, args: Value) -> AppResult<String> {
        let result = self
            .request("tools/call", Some(json!({"name": name, "arguments": args})))
            .await?;
        if let Some(content) = result["content"].as_array() {
            let parts: Vec<&str> = content
                .iter()
                .filter_map(|c| {
                    if c["type"].as_str() == Some("text") {
                        c["text"].as_str()
                    } else {
                        None
                    }
                })
                .collect();
            return Ok(parts.join("\n"));
        }
        Ok(result.to_string())
    }
}
