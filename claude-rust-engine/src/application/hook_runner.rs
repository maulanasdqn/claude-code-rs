use claude_rust_config::HooksConfig;
use claude_rust_config::HookEntry;
use tokio::process::Command;

struct Outcome {
    exit_code: i32,
    stdout: String,
}

async fn run_cmd(command: &str, env_vars: &[(&str, &str)]) -> Outcome {
    let (shell, flag) = claude_rust_config::shell_command();
    let mut cmd = Command::new(shell);
    cmd.arg(flag).arg(command);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    match cmd.output().await {
        Ok(out) => Outcome {
            exit_code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        },
        Err(_) => Outcome { exit_code: 1, stdout: String::new() },
    }
}

fn tool_matches(entry: &HookEntry, tool_name: &str) -> bool {
    match &entry.matcher {
        None => true,
        Some(m) if m.is_empty() => true,
        Some(m) => tool_name.contains(m.as_str()),
    }
}

pub struct PreToolDecision {
    pub blocked: bool,
    pub reason: String,
    pub output: String,
}

pub async fn run_pre_tool_use(hooks: &HooksConfig, tool_name: &str, input_json: &str) -> PreToolDecision {
    let env = [("CLAUDE_TOOL_NAME", tool_name), ("CLAUDE_TOOL_INPUT", input_json)];
    for entry in hooks.pre_tool_use.iter().filter(|e| tool_matches(e, tool_name)) {
        let out = run_cmd(&entry.command, &env).await;
        if out.exit_code != 0 {
            let reason = if out.stdout.is_empty() {
                format!("blocked by hook (exit {})", out.exit_code)
            } else {
                out.stdout
            };
            return PreToolDecision { blocked: true, reason, output: String::new() };
        }
        if !out.stdout.is_empty() {
            return PreToolDecision { blocked: false, reason: String::new(), output: out.stdout };
        }
    }
    PreToolDecision { blocked: false, reason: String::new(), output: String::new() }
}

pub async fn run_post_tool_use(
    hooks: &HooksConfig,
    tool_name: &str,
    input_json: &str,
    tool_output: &str,
) -> String {
    let env = [
        ("CLAUDE_TOOL_NAME", tool_name),
        ("CLAUDE_TOOL_INPUT", input_json),
        ("CLAUDE_TOOL_OUTPUT", tool_output),
    ];
    let mut out = String::new();
    for entry in hooks.post_tool_use.iter().filter(|e| tool_matches(e, tool_name)) {
        let result = run_cmd(&entry.command, &env).await;
        if !result.stdout.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str(&result.stdout);
        }
    }
    out
}

pub async fn run_stop_hooks(hooks: &HooksConfig) -> String {
    let env: [(&str, &str); 0] = [];
    let mut out = String::new();
    for entry in &hooks.stop {
        let result = run_cmd(&entry.command, &env).await;
        if !result.stdout.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str(&result.stdout);
        }
    }
    out
}

pub async fn run_session_start_hooks(hooks: &HooksConfig) -> String {
    let env: [(&str, &str); 0] = [];
    let mut out = String::new();
    for entry in &hooks.session_start {
        let result = run_cmd(&entry.command, &env).await;
        if !result.stdout.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str(&result.stdout);
        }
    }
    out
}
