use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub permissions: PermissionSettings,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub max_turns: Option<usize>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub effort: Option<String>,
    /// When true, the assistant may attribute commits to itself (e.g. add a
    /// `Co-Authored-By:` trailer). Defaults to false — commits stay clean.
    #[serde(default)]
    pub commit_attribution: bool,
    /// Intern definitions — each becomes a separate `delegate_to_<name>` tool
    /// the senior model can call to hand off a focused subtask.
    #[serde(default)]
    pub interns: Vec<InternConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InternConfig {
    /// Short identifier — becomes part of the tool name (`delegate_to_<name>`).
    pub name: String,
    /// Provider shorthand: "deepseek" | "openrouter" | "openai" | "custom".
    /// Determines the default base_url and api_key env var.
    pub provider: String,
    /// Model id passed to the provider (e.g. "deepseek-chat",
    /// "anthropic/claude-haiku-4.5", "qwen/qwen3-coder").
    pub model: String,
    /// Optional one-liner the senior model sees in the tool description.
    /// Use this to say what the intern is good at.
    #[serde(default)]
    pub description: Option<String>,
    /// Override base_url. Required when provider = "custom".
    #[serde(default)]
    pub base_url: Option<String>,
    /// Override the env var name to read the api key from.
    /// Defaults to the provider's standard key (DEEPSEEK_API_KEY, etc).
    #[serde(default)]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSettings {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookEntry {
    #[serde(default)]
    pub matcher: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default, rename = "PreToolUse")]
    pub pre_tool_use: Vec<HookEntry>,
    #[serde(default, rename = "PostToolUse")]
    pub post_tool_use: Vec<HookEntry>,
    #[serde(default, rename = "Stop")]
    pub stop: Vec<HookEntry>,
    #[serde(default, rename = "SessionStart")]
    pub session_start: Vec<HookEntry>,
}
