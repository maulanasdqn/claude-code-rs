use stynx_code_types::EngineEvent;
use super::{ConversationState, DiffLine, DiffLineKind, DisplayMessage, DisplayToolUse, InputState, ModalState, ToastState, ToolUseStatus};

/// USD price per million tokens `(input, output)` for a given model.
///
/// Matched on substrings so provider prefixes / date suffixes still resolve
/// (e.g. `deepseek-chat`, `claude-opus-4-20250514`). Falls back to Claude
/// Sonnet rates for unknown models.
fn model_pricing(model: &str) -> (f64, f64) {
    let m = model.to_ascii_lowercase();
    match () {
        // DeepSeek (per their published API pricing, cache-miss rates)
        _ if m.contains("deepseek-reasoner") || m.contains("deepseek-r1") => (0.55, 2.19),
        _ if m.contains("deepseek") => (0.27, 1.10),
        // Anthropic
        _ if m.contains("opus") => (15.0, 75.0),
        _ if m.contains("haiku") => (0.80, 4.0),
        _ if m.contains("sonnet") => (3.0, 15.0),
        // OpenAI (common ones)
        _ if m.contains("gpt-4o-mini") => (0.15, 0.60),
        _ if m.contains("gpt-4o") => (2.50, 10.0),
        // Unknown → assume Sonnet-class so the figure is non-zero but sane.
        _ => (3.0, 15.0),
    }
}

#[derive(Clone)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub pinned: bool,
}

pub struct SidebarState {
    pub visible: bool,
    pub title: String,
    pub session_id: String,
    pub version: String,
    pub sessions: Vec<SessionSummary>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            visible: true,
            title: "New session".to_string(),
            session_id: String::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            sessions: Vec::new(),
        }
    }
}

impl Default for SidebarState {
    fn default() -> Self { Self::new() }
}

pub struct AppState {
    pub input: InputState,
    pub conversation: ConversationState,
    pub modal: ModalState,
    pub sidebar: SidebarState,
    pub toasts: ToastState,
    pub model_name: String,
    pub permission_mode: String,
    pub total_cost: f64,
    pub git_branch: Option<String>,
    pub cwd: String,
    pub is_streaming: bool,
    pub is_paused: bool,
    pub spinner_frame: usize,
    pub spinner_tick: u8,
    pub total_input: u64,
    pub total_output: u64,
    pub recent_models: Vec<String>,
    pub tool_details: bool,

    pub live_thinking: String,

    pub sub_agents: Vec<(String, String)>,

    pub last_summary: Option<Vec<String>>,

    pub tool_history: ToolHistoryState,

    pub is_pending: bool,
    pub elapsed_secs: u64,
    pub stale_warned: bool,
}

#[derive(Default)]
pub struct ToolHistoryState {
    pub selected: Option<usize>,
    pub scroll: usize,
    pub focused: bool,
    pub detail_open: bool,
}

impl AppState {
    pub fn push_recent_model(&mut self, id: &str) {
        self.recent_models.retain(|m| m != id);
        self.recent_models.insert(0, id.to_string());
        if self.recent_models.len() > 8 {
            self.recent_models.truncate(8);
        }
    }

    pub fn cycle_recent_model(&mut self) -> Option<String> {
        if self.recent_models.len() < 2 {
            return None;
        }
        let next = self.recent_models.remove(1);
        self.recent_models.insert(0, next.clone());
        Some(next)
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            conversation: ConversationState::new(),
            modal: ModalState::new(),
            sidebar: SidebarState::new(),
            toasts: ToastState::new(),
            model_name: String::from("claude-sonnet-4-20250514"),
            permission_mode: String::from("Normal"),
            total_cost: 0.0,
            git_branch: None,
            cwd: std::env::current_dir().ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_default(),
            is_streaming: false,
            is_paused: false,
            spinner_frame: 0,
            spinner_tick: 0,
            total_input: 0,
            total_output: 0,
            recent_models: Vec::new(),
            tool_details: true,
            live_thinking: String::new(),
            sub_agents: Vec::new(),
            last_summary: None,
            tool_history: ToolHistoryState::default(),
            is_pending: false,
            elapsed_secs: 0,
            stale_warned: false,
        }
    }

    pub fn push_user_message(&mut self, text: impl Into<String>) {
        self.last_summary = None;
        self.is_paused = false;
        self.conversation.messages.push(DisplayMessage {
            role: "user".to_string(),
            content: text.into(),
            thinking: String::new(),
            tool_uses: Vec::new(),
            is_streaming: false,
        });
        self.conversation.auto_scroll = true;
    }

    pub fn push_system_message(&mut self, text: impl Into<String>) {
        self.conversation.messages.push(DisplayMessage {
            role: "system".to_string(),
            content: text.into(),
            thinking: String::new(),
            tool_uses: Vec::new(),
            is_streaming: false,
        });
        self.conversation.auto_scroll = true;
    }

    pub fn apply_engine_event(&mut self, event: EngineEvent) {
        self.is_pending = false;
        match event {
            EngineEvent::TextDelta(text) => {
                self.is_streaming = true;
                match self.conversation.messages.last_mut() {
                    Some(m) if m.role == "assistant" && m.is_streaming => m.content.push_str(&text),
                    _ => self.conversation.messages.push(DisplayMessage {
                        role: "assistant".to_string(), content: text,
                        thinking: String::new(), tool_uses: Vec::new(), is_streaming: true,
                    }),
                }
            }
            EngineEvent::ThinkingDelta(text) => {
                self.is_streaming = true;
                self.live_thinking.push_str(&text);
                // Make sure a streaming assistant bubble exists so the "Stynx"
                // header and its thinking indicator render while only reasoning
                // (no text/tools yet) has streamed in.
                if !matches!(self.conversation.messages.last(),
                    Some(m) if m.role == "assistant" && m.is_streaming)
                {
                    self.conversation.messages.push(DisplayMessage {
                        role: "assistant".to_string(), content: String::new(),
                        thinking: String::new(), tool_uses: Vec::new(), is_streaming: true,
                    });
                }
            }
            EngineEvent::ToolStart { name, .. } => {
                self.is_streaming = true;
                let tool = DisplayToolUse {
                    name,
                    status: ToolUseStatus::Running,
                    output_preview: String::new(),
                    input_json: String::new(),
                    input_summary: String::new(),
                    output_excerpt: Vec::new(),
                    diff: Vec::new(),
                    sub_progress: Vec::new(),
                };
                match self.conversation.messages.last_mut().filter(|m| m.role == "assistant") {
                    Some(m) => m.tool_uses.push(tool),
                    None => self.conversation.messages.push(DisplayMessage {
                        role: "assistant".to_string(), content: String::new(),
                        thinking: String::new(), tool_uses: vec![tool], is_streaming: true,
                    }),
                }
            }
            EngineEvent::ToolInput { json_chunk } => {
                if let Some(m) = self.conversation.messages.last_mut() {
                    if let Some(t) = m.tool_uses.iter_mut().rev()
                        .find(|t| t.status == ToolUseStatus::Running)
                    {
                        t.input_json.push_str(&json_chunk);
                        t.input_summary = summarize_tool_input(&t.name, &t.input_json);
                    }
                }
            }
            EngineEvent::ToolResult { name, output, is_error } => {
                let clean_output = crate::util::strip_ansi(&output);
                let preview_limit = if is_error { 400 } else { 80 };
                if let Some(m) = self.conversation.messages.last_mut() {
                    if let Some(t) = m.tool_uses.iter_mut().rev()
                        .find(|t| t.name == name && t.status == ToolUseStatus::Running) {
                        t.status = if is_error { ToolUseStatus::Error } else { ToolUseStatus::Completed };
                        t.output_preview = clean_output.lines().next().unwrap_or("").chars().take(preview_limit).collect();
                        if t.input_summary.is_empty() {
                            t.input_summary = summarize_tool_input(&t.name, &t.input_json);
                        }
                        t.output_excerpt = excerpt_lines(&clean_output, 6, 200);
                        if t.name == "file_edit" || t.name == "file_write" {
                            t.diff = build_diff_for(&t.name, &t.input_json);
                        }

                        if matches!(t.name.as_str(), "read" | "grep" | "glob") && !is_error {
                            let n = clean_output.lines().filter(|l| !l.trim().is_empty()).count();
                            if n > 0 && !t.input_summary.contains("(")  {
                                t.input_summary = format!("{}  ({n} lines)", t.input_summary);
                            }
                        }
                    }
                }
                if is_error {
                    self.conversation.messages.push(DisplayMessage {
                        role: "error".to_string(),
                        content: format!("{name}: {clean_output}"),
                        thinking: String::new(),
                        tool_uses: Vec::new(),
                        is_streaming: false,
                    });
                    tracing::error!(tool = %name, output = %clean_output, "tool returned error");
                }
            }
            EngineEvent::TurnComplete => {
                self.is_streaming = false;
                let tool_summary = self.conversation.messages.last().and_then(|m| {
                    if m.role != "assistant" { return None; }
                    if m.tool_uses.is_empty() { return None; }
                    let parts: Vec<String> = m.tool_uses.iter().map(|t| {
                        let pretty = match t.name.as_str() {
                            "bash" => "Bash".into(),
                            "read" => "Read".into(),
                            "file_write" => "Write".into(),
                            "file_edit" => "Edit".into(),
                            "glob" => "Glob".into(),
                            "grep" => "Grep".into(),
                            "web_fetch" => "WebFetch".into(),
                            "web_search" => "WebSearch".into(),
                            "todo_write" => "TodoWrite".into(),
                            "todo_read" => "TodoRead".into(),
                            "ask_user_question" => "AskUser".into(),
                            "agent" => "Agent".into(),
                            other => {
                                let mut s = other.replace('_', " ");
                                s = s.split_whitespace()
                                    .map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() })
                                    .collect::<Vec<_>>().join("");
                                s
                            }
                        };
                        if t.input_summary.is_empty() {
                            pretty
                        } else {
                            format!("{}({})", pretty, t.input_summary)
                        }
                    }).collect();
                    if parts.is_empty() { None } else { Some(parts) }
                });
                if let Some(m) = self.conversation.messages.last_mut() {
                    m.is_streaming = false;
                    if !self.live_thinking.is_empty() && m.role == "assistant" {
                        if m.thinking.is_empty() {
                            m.thinking = std::mem::take(&mut self.live_thinking);
                        } else {
                            m.thinking.push_str(&self.live_thinking);
                            self.live_thinking.clear();
                        }
                    }
                }
                self.live_thinking.clear();
                if let Some(summary) = tool_summary {
                    self.last_summary = Some(summary);
                    self.conversation.auto_scroll = true;
                }
            }
            EngineEvent::Usage { input_tokens, output_tokens } => {
                if input_tokens > 0 { self.total_input += input_tokens; }
                if output_tokens > 0 { self.total_output += output_tokens; }
                let (in_price, out_price) = model_pricing(&self.model_name);
                self.total_cost = (self.total_input as f64 * in_price
                    + self.total_output as f64 * out_price) / 1_000_000.0;
            }
            EngineEvent::Error(e) => {
                self.is_streaming = false;
                self.conversation.messages.push(DisplayMessage {
                    role: "error".to_string(), content: e,
                    thinking: String::new(), tool_uses: Vec::new(), is_streaming: false,
                });
            }
            EngineEvent::ModeChanged { mode } => {
                self.permission_mode = mode.label().to_string();
            }
            EngineEvent::SubAgentProgress { label, summary } => {
                match self.sub_agents.iter_mut().find(|(l, _)| l == &label) {
                    Some((_, s)) => *s = summary.clone(),
                    None => self.sub_agents.push((label.clone(), summary.clone())),
                }
                if let Some(m) = self.conversation.messages.last_mut() {
                    if let Some(t) = m.tool_uses.iter_mut().rev()
                        .find(|t| t.status == ToolUseStatus::Running
                            && (t.name == "agent" || t.name == "explore"
                                || t.name.starts_with("delegate_to_")))
                    {
                        t.sub_progress.push(format!("{label}: {summary}"));
                        if t.sub_progress.len() > 50 {
                            let drop = t.sub_progress.len() - 50;
                            t.sub_progress.drain(0..drop);
                        }
                    }
                }
            }
            EngineEvent::SubAgentDone { label } => {
                self.sub_agents.retain(|(l, _)| l != &label);
            }
            _ => {}
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

fn try_parse(json: &str) -> Option<serde_json::Value> {
    if json.trim().is_empty() { return None; }
    serde_json::from_str(json).ok()
}

fn shorten(s: &str, max: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}

pub fn summarize_tool_input(tool: &str, json: &str) -> String {
    let parsed = try_parse(json);
    let get = |k: &str| -> String {
        parsed
            .as_ref()
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    match tool {
        "bash" => {
            if parsed.as_ref().and_then(|v| v.get("list")).and_then(|v| v.as_bool()).unwrap_or(false) {
                return "list background processes".to_string();
            }
            if let Some(h) = parsed.as_ref().and_then(|v| v.get("kill")).and_then(|v| v.as_str()) {
                return format!("kill {h}");
            }
            if let Some(h) = parsed.as_ref().and_then(|v| v.get("status")).and_then(|v| v.as_str()) {
                return format!("status {h}");
            }
            let cmd = get("command");
            if cmd.is_empty() { return String::new(); }
            let bg = parsed.as_ref().and_then(|v| v.get("background")).and_then(|v| v.as_bool()).unwrap_or(false);
            let suffix = if bg { "  &" } else { "" };
            format!("$ {}{suffix}", shorten(first_line(&cmd), 140))
        }
        "read" => {
            let path = get("file_path");
            if path.is_empty() { return String::new(); }
            let offset = parsed.as_ref().and_then(|v| v.get("offset")).and_then(|v| v.as_u64());
            let limit = parsed.as_ref().and_then(|v| v.get("limit")).and_then(|v| v.as_u64());
            match (offset, limit) {
                (Some(o), Some(l)) => format!("{path}:{o}-{}", o + l),
                (Some(o), None) => format!("{path}:{o}-"),
                _ => path,
            }
        }
        "file_write" => {
            let path = get("file_path");
            let content_len = parsed
                .as_ref()
                .and_then(|v| v.get("content"))
                .and_then(|v| v.as_str())
                .map(|s| s.lines().count())
                .unwrap_or(0);
            if path.is_empty() { return String::new(); }
            if content_len > 0 { format!("{path}  ({content_len} lines)") } else { path }
        }
        "file_edit" => {
            let path = get("file_path");
            if path.is_empty() { return String::new(); }
            path
        }
        "glob" => {
            let pattern = get("pattern");
            if pattern.is_empty() { return String::new(); }
            shorten(&pattern, 140)
        }
        "grep" => {
            let pattern = get("pattern");
            let path = get("path");
            let mut s = shorten(&pattern, 100);
            if !path.is_empty() {
                s.push_str(" in ");
                s.push_str(&shorten(&path, 40));
            }
            s
        }
        "web_fetch" | "web_search" => {
            let url = get("url");
            let q = get("query");
            if !url.is_empty() { shorten(&url, 140) } else { shorten(&q, 140) }
        }
        "delegate_to_intern" => {
            let task = get("task");
            shorten(first_line(&task), 140)
        }
        "agent" | "explore" => {
            let task = get("task");
            shorten(first_line(&task), 140)
        }
        "todo_write" | "todo_read" => String::new(),
        _ => {

            parsed
                .as_ref()
                .and_then(|v| v.as_object())
                .and_then(|m| m.values().find_map(|v| v.as_str()))
                .map(|s| shorten(first_line(s), 120))
                .unwrap_or_default()
        }
    }
}

pub fn build_diff_for(tool: &str, input_json: &str) -> Vec<DiffLine> {
    let Some(v) = try_parse(input_json) else { return Vec::new(); };
    let max_lines = 14usize;
    let context_lines = 2usize;

    if tool == "file_write" {
        let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
        return content
            .lines()
            .take(max_lines)
            .map(|l| DiffLine {
                kind: DiffLineKind::Added,
                text: l.to_string(),
            })
            .collect();
    }

    let old_s = v.get("old_string").and_then(|s| s.as_str()).unwrap_or("");
    let new_s = v.get("new_string").and_then(|s| s.as_str()).unwrap_or("");

    let old_lines: Vec<&str> = old_s.split('\n').collect();
    let new_lines: Vec<&str> = new_s.split('\n').collect();

    let mut p = 0;
    while p < old_lines.len() && p < new_lines.len() && old_lines[p] == new_lines[p] {
        p += 1;
    }

    let mut s = 0;
    while s < old_lines.len() - p && s < new_lines.len() - p
        && old_lines[old_lines.len() - 1 - s] == new_lines[new_lines.len() - 1 - s]
    {
        s += 1;
    }

    let mut out: Vec<DiffLine> = Vec::new();

    let ctx_start = p.saturating_sub(context_lines);
    for line in &old_lines[ctx_start..p] {
        out.push(DiffLine { kind: DiffLineKind::Context, text: line.to_string() });
    }

    for line in &old_lines[p..old_lines.len() - s] {
        out.push(DiffLine { kind: DiffLineKind::Removed, text: line.to_string() });
        if out.len() >= max_lines { return out; }
    }

    for line in &new_lines[p..new_lines.len() - s] {
        out.push(DiffLine { kind: DiffLineKind::Added, text: line.to_string() });
        if out.len() >= max_lines { return out; }
    }

    let ctx_end_start = old_lines.len() - s;
    let ctx_end_stop = (ctx_end_start + context_lines).min(old_lines.len());
    for line in &old_lines[ctx_end_start..ctx_end_stop] {
        out.push(DiffLine { kind: DiffLineKind::Context, text: line.to_string() });
        if out.len() >= max_lines { return out; }
    }

    out
}

pub fn excerpt_lines(text: &str, max_lines: usize, max_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(max_lines + 1)
        .map(|l| {
            if l.chars().count() > max_width {
                let mut s: String = l.chars().take(max_width.saturating_sub(1)).collect();
                s.push('…');
                s
            } else {
                l.to_string()
            }
        })
        .collect();
    let total = text.lines().filter(|l| !l.trim().is_empty()).count();
    if total > max_lines {
        lines.truncate(max_lines);
        lines.push(format!("… +{} more lines", total - max_lines));
    }
    lines
}
