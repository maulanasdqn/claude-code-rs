use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct WebFetchTool;

const MAX_RESPONSE_SIZE: usize = 100_000;
const TIMEOUT_SECS: u64 = 30;

#[async_trait::async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch the content of a web page at the given URL. Returns the page text content."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                }
            },
            "required": ["url"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }
    fn is_open_world(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let url = input
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| AppError::Tool("missing 'url' field".into()))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .build()
            .map_err(|e| AppError::Tool(format!("failed to create HTTP client: {e}")))?;

        let response = client
            .get(url)
            .header("User-Agent", "claude-code-rs/0.2.0")
            .send()
            .await
            .map_err(|e| AppError::Tool(format!("fetch failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::Tool(format!("HTTP {status} for {url}")));
        }

        let body = response
            .text()
            .await
            .map_err(|e| AppError::Tool(format!("failed to read response body: {e}")))?;

        // Strip HTML tags for readability
        let text = strip_html_tags(&body);

        // Truncate if too large
        if text.len() > MAX_RESPONSE_SIZE {
            Ok(format!(
                "{}...\n(truncated at {}KB)",
                &text[..MAX_RESPONSE_SIZE],
                MAX_RESPONSE_SIZE / 1000
            ))
        } else {
            Ok(text)
        }
    }
}

/// Basic HTML tag stripping via simple state machine.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_was_whitespace = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if !in_tag && chars[i] == '<' {
            // Check for script/style tags
            if i + 7 < len && &lower[i..i + 7] == "<script" {
                in_script = true;
            }
            if i + 6 < len && &lower[i..i + 6] == "<style" {
                in_style = true;
            }
            if in_script && i + 9 <= len && &lower[i..i + 9] == "</script>" {
                in_script = false;
                i += 9;
                continue;
            }
            if in_style && i + 8 <= len && &lower[i..i + 8] == "</style>" {
                in_style = false;
                i += 8;
                continue;
            }
            in_tag = true;
            i += 1;
            continue;
        }

        if in_tag {
            if chars[i] == '>' {
                in_tag = false;
            }
            i += 1;
            continue;
        }

        if in_script || in_style {
            i += 1;
            continue;
        }

        // Decode common entities
        if chars[i] == '&' {
            if i + 4 < len && &html[i..i + 4] == "&lt;" {
                result.push('<');
                last_was_whitespace = false;
                i += 4;
                continue;
            }
            if i + 4 < len && &html[i..i + 4] == "&gt;" {
                result.push('>');
                last_was_whitespace = false;
                i += 4;
                continue;
            }
            if i + 5 < len && &html[i..i + 5] == "&amp;" {
                result.push('&');
                last_was_whitespace = false;
                i += 5;
                continue;
            }
            if i + 6 < len && &html[i..i + 6] == "&nbsp;" {
                result.push(' ');
                last_was_whitespace = true;
                i += 6;
                continue;
            }
        }

        let c = chars[i];
        if c.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
        i += 1;
    }

    // Collapse multiple blank lines
    let mut cleaned = String::new();
    let mut blank_count = 0;
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                cleaned.push('\n');
            }
        } else {
            blank_count = 0;
            cleaned.push_str(trimmed);
            cleaned.push('\n');
        }
    }

    let _ = lower_chars; // suppress unused warning
    cleaned.trim().to_string()
}
