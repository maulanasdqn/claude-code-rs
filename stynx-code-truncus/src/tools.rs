use std::sync::Arc;

use serde_json::{json, Value};
use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};

use crate::client::ApiClient;
use crate::config::Config;
use crate::dto::{Lesson, SearchHit, SessionMeta};
use crate::project::project_from_cwd;
use crate::util::{fmt_date, truncate};

/// The Truncus memory search tools, scoped to `cwd`'s project by default.
/// Returns an empty list when Truncus is not configured, so the tools simply do
/// not appear rather than failing at call time.
pub fn memory_tools(cwd: &str) -> Vec<Arc<dyn Tool>> {
    let Some(cfg) = Config::load() else {
        return Vec::new();
    };
    let client = Arc::new(ApiClient::new(&cfg));
    let project = project_from_cwd(cwd);
    vec![
        Arc::new(MemorySearchTool { client: client.clone(), project: project.clone() }),
        Arc::new(RecentSessionsTool { client: client.clone(), project: project.clone() }),
        Arc::new(GetSessionTool { client: client.clone() }),
        Arc::new(LessonsTool { client: client.clone(), project: project.clone() }),
        Arc::new(KnowledgeSearchTool { client, project }),
    ]
}

fn api_err(e: String) -> AppError {
    AppError::Tool(format!("truncus: {e}"))
}

/// Resolve the project filter: `all: true` searches everywhere, an explicit
/// `project` overrides, otherwise the tool's default (current) project is used.
fn resolve_project(input: &Value, default: &str) -> Option<String> {
    if input.get("all").and_then(Value::as_bool).unwrap_or(false) {
        return None;
    }
    match input.get("project").and_then(Value::as_str) {
        Some(p) if !p.trim().is_empty() => Some(p.to_string()),
        _ => Some(default.to_string()),
    }
}

fn limit_of(input: &Value, default: usize) -> usize {
    input
        .get("limit")
        .and_then(Value::as_u64)
        .map(|n| (n as usize).clamp(1, 50))
        .unwrap_or(default)
}

fn required_str<'a>(input: &'a Value, key: &str) -> AppResult<&'a str> {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Tool(format!("truncus: `{key}` is required")))
}

fn search_props() -> Value {
    json!({
        "query": { "type": "string", "description": "Natural-language search query." },
        "limit": { "type": "integer", "description": "Max results (default 8, max 50)." },
        "project": { "type": "string", "description": "Scope to a project name. Defaults to the current project." },
        "all": { "type": "boolean", "description": "Search across all projects instead of just the current one." }
    })
}

fn render_hits(hits: &[SearchHit]) -> String {
    if hits.is_empty() {
        return "No matches.".to_string();
    }
    hits.iter()
        .map(|h| {
            format!(
                "[{} · {} · {} · score {:.2}]\n{}",
                h.project,
                h.kind,
                fmt_date(h.ended_at),
                h.score,
                truncate(&h.text, 800)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_lessons(lessons: &[Lesson]) -> String {
    if lessons.is_empty() {
        return "No lessons recorded yet.".to_string();
    }
    lessons
        .iter()
        .map(|l| {
            format!(
                "- [{}] {} (seen {}×, confidence {:.2})\n  {}",
                l.category, l.title, l.times_seen, l.confidence, l.insight
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_session(m: &SessionMeta) -> String {
    let summary = m.summary.clone().unwrap_or_else(|| "(no summary)".to_string());
    format!(
        "session {} · {} · {} · {} [{}] · {} chunks\n{}",
        m.id,
        m.project,
        m.machine,
        fmt_date(m.ended_at),
        m.status,
        m.chunk_count,
        summary
    )
}

pub struct MemorySearchTool {
    client: Arc<ApiClient>,
    project: String,
}

#[async_trait::async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str { "memory_search" }
    fn description(&self) -> &str {
        "Semantic search over your past session summaries and conversation chunks (Truncus memory). Use to recall prior work, decisions, or how something was done before."
    }
    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": search_props(), "required": ["query"] })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let query = required_str(&input, "query")?;
        let project = resolve_project(&input, &self.project);
        let resp = self
            .client
            .search(query, project.as_deref(), limit_of(&input, 8))
            .await
            .map_err(api_err)?;
        Ok(render_hits(&resp.hits))
    }
}

pub struct KnowledgeSearchTool {
    client: Arc<ApiClient>,
    project: String,
}

#[async_trait::async_trait]
impl Tool for KnowledgeSearchTool {
    fn name(&self) -> &str { "knowledge_search" }
    fn description(&self) -> &str {
        "Semantic search over the project's knowledge base (notes synced from a vault). Use to pull reference material on demand instead of loading whole documents."
    }
    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": search_props(), "required": ["query"] })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let query = required_str(&input, "query")?;
        let project = resolve_project(&input, &self.project);
        let resp = self
            .client
            .knowledge(query, project.as_deref(), limit_of(&input, 8))
            .await
            .map_err(api_err)?;
        Ok(render_hits(&resp.hits))
    }
}

pub struct LessonsTool {
    client: Arc<ApiClient>,
    project: String,
}

#[async_trait::async_trait]
impl Tool for LessonsTool {
    fn name(&self) -> &str { "lessons" }
    fn description(&self) -> &str {
        "List durable lessons Truncus has distilled for this project (pitfalls, fixes, preferences, conventions), most reinforced first."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Max lessons (default 15, max 50)." },
                "project": { "type": "string", "description": "Scope to a project name. Defaults to the current project." },
                "all": { "type": "boolean", "description": "List lessons across all projects." }
            }
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let project = resolve_project(&input, &self.project);
        let resp = self
            .client
            .lessons(project.as_deref(), limit_of(&input, 15))
            .await
            .map_err(api_err)?;
        Ok(render_lessons(&resp.lessons))
    }
}

pub struct RecentSessionsTool {
    client: Arc<ApiClient>,
    project: String,
}

#[async_trait::async_trait]
impl Tool for RecentSessionsTool {
    fn name(&self) -> &str { "recent_sessions" }
    fn description(&self) -> &str {
        "List your most recent captured sessions (id, project, date, summary). Use to see what you worked on recently."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Max sessions (default 10, max 50)." },
                "project": { "type": "string", "description": "Scope to a project name. Defaults to the current project." },
                "all": { "type": "boolean", "description": "List sessions across all projects." }
            }
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let project = resolve_project(&input, &self.project);
        let resp = self
            .client
            .sessions(project.as_deref(), limit_of(&input, 10))
            .await
            .map_err(api_err)?;
        if resp.sessions.is_empty() {
            return Ok("No sessions captured yet.".to_string());
        }
        Ok(resp
            .sessions
            .iter()
            .map(render_session)
            .collect::<Vec<_>>()
            .join("\n\n"))
    }
}

pub struct GetSessionTool {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl Tool for GetSessionTool {
    fn name(&self) -> &str { "get_session" }
    fn description(&self) -> &str {
        "Fetch one captured session by id, with its distilled summary."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "The session id to fetch." }
            },
            "required": ["session_id"]
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let id = required_str(&input, "session_id")?;
        let meta = self.client.session(id).await.map_err(api_err)?;
        Ok(render_session(&meta))
    }
}
