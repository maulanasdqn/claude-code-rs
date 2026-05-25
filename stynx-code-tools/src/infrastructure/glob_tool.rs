use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

pub struct GlobTool;

#[async_trait::async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern. Returns paths sorted by modification time (newest first)."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match files (e.g. \"**/*.rs\", \"src/**/*.ts\")"
                },
                "path": {
                    "type": "string",
                    "description": "Base directory to search in (defaults to current working directory)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: true, is_read: false, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'pattern' field".into()))?;

        let base = input
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| ".".into())
            });

        tracing::info!(pattern, base, "globbing files");

        let full_pattern = if pattern.starts_with('/') {
            pattern.to_string()
        } else {
            format!("{base}/{pattern}")
        };

        let entries = glob::glob(&full_pattern)
            .map_err(|e| AppError::Tool(format!("invalid glob pattern: {e}")))?;

        let mut files: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();
        for entry in entries {
            if let Ok(path) = entry
                && path.is_file() {
                    let mtime = path
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    files.push((path, mtime));
                }
        }

        files.sort_by(|a, b| b.1.cmp(&a.1));
        files.truncate(200);

        if files.is_empty() {
            return Ok("No files matched.".into());
        }

        let result: Vec<String> = files.iter().map(|(p, _)| p.display().to_string()).collect();
        Ok(format!("{} files matched:\n{}", result.len(), result.join("\n")))
    }
}
