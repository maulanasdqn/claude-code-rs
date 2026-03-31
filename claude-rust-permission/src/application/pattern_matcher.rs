use serde_json::Value;

/// A parsed permission rule, e.g. "bash(git *)" → tool="bash", pattern=Some("git *")
#[derive(Debug, Clone)]
pub struct PermissionRule {
    pub tool: String,
    pub pattern: Option<String>,
}

/// Parse a permission rule string like "bash(git *)", "read", "glob".
pub fn parse_rule(rule: &str) -> PermissionRule {
    let rule = rule.trim();
    if let Some(paren_start) = rule.find('(') {
        if rule.ends_with(')') {
            let tool = rule[..paren_start].to_string();
            let pattern = rule[paren_start + 1..rule.len() - 1].to_string();
            return PermissionRule {
                tool,
                pattern: Some(pattern),
            };
        }
    }
    PermissionRule {
        tool: rule.to_string(),
        pattern: None,
    }
}

/// Check if a permission rule matches a given tool invocation.
pub fn rule_matches(rule: &PermissionRule, tool_name: &str, input: &Value) -> bool {
    if rule.tool != tool_name {
        return false;
    }

    let Some(pattern) = &rule.pattern else {
        // Tool name only match — matches any input
        return true;
    };

    // Get the relevant input field based on tool type
    let field_value = match tool_name {
        "bash" => input.get("command").and_then(|v| v.as_str()),
        "file_write" | "file_edit" => input.get("file_path").and_then(|v| v.as_str()),
        "read" => input.get("file_path").and_then(|v| v.as_str()),
        "web_fetch" => input.get("url").and_then(|v| v.as_str()),
        _ => {
            // For other tools, try to match against the full input JSON string
            None
        }
    };

    let Some(value) = field_value else {
        return false;
    };

    glob_match(pattern, value)
}

/// Simple glob matching: `*` matches any sequence of chars.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcards — exact match
        return pattern == text;
    }

    let mut pos = 0;

    // First part must match from the start
    if !parts[0].is_empty() {
        if !text.starts_with(parts[0]) {
            return false;
        }
        pos = parts[0].len();
    }

    // Middle parts must appear in order
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }

    // Last part must match at the end
    let last = parts[parts.len() - 1];
    if !last.is_empty() {
        text[pos..].ends_with(last)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_rule_simple() {
        let rule = parse_rule("read");
        assert_eq!(rule.tool, "read");
        assert!(rule.pattern.is_none());
    }

    #[test]
    fn test_parse_rule_with_pattern() {
        let rule = parse_rule("bash(git *)");
        assert_eq!(rule.tool, "bash");
        assert_eq!(rule.pattern.as_deref(), Some("git *"));
    }

    #[test]
    fn test_rule_matches_tool_only() {
        let rule = parse_rule("read");
        assert!(rule_matches(&rule, "read", &json!({"file_path": "/any/path"})));
        assert!(!rule_matches(&rule, "bash", &json!({"command": "ls"})));
    }

    #[test]
    fn test_rule_matches_bash_pattern() {
        let rule = parse_rule("bash(git *)");
        assert!(rule_matches(&rule, "bash", &json!({"command": "git status"})));
        assert!(rule_matches(&rule, "bash", &json!({"command": "git diff --cached"})));
        assert!(!rule_matches(&rule, "bash", &json!({"command": "rm -rf /"})));
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("git *", "git status"));
        assert!(glob_match("git *", "git diff --cached"));
        assert!(!glob_match("git *", "rm -rf"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.py"));
    }
}
