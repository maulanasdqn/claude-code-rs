use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct WebSearchTool;

const TIMEOUT_SECS: u64 = 15;

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web using DuckDuckGo. Returns a list of results with titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "number",
                    "description": "Maximum number of results to return (default: 10)"
                }
            },
            "required": ["query"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let query = input
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| AppError::Tool("missing 'query' field".into()))?;

        let max_results = input
            .get("max_results")
            .and_then(|m| m.as_u64())
            .unwrap_or(10) as usize;

        let encoded_query = urlencoded(query);
        let url = format!("https://html.duckduckgo.com/html/?q={encoded_query}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .build()
            .map_err(|e| AppError::Tool(format!("failed to create HTTP client: {e}")))?;

        let response = client
            .get(&url)
            .header("User-Agent", "claude-code-rs/0.2.0")
            .send()
            .await
            .map_err(|e| AppError::Tool(format!("search request failed: {e}")))?;

        let body = response
            .text()
            .await
            .map_err(|e| AppError::Tool(format!("failed to read search results: {e}")))?;

        let results = parse_duckduckgo_results(&body, max_results);

        if results.is_empty() {
            return Ok(format!("No results found for: {query}"));
        }

        let mut output = format!("Search results for: {query}\n\n");
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            ));
        }

        Ok(output)
    }
}

struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

fn parse_duckduckgo_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // DuckDuckGo HTML results have class="result__a" for title links
    // and class="result__snippet" for snippets
    let mut pos = 0;
    while results.len() < max {
        // Find result link
        let link_marker = "class=\"result__a\"";
        let Some(link_start) = html[pos..].find(link_marker) else {
            break;
        };
        let link_start = pos + link_start;

        // Extract href
        let href_area = &html[link_start.saturating_sub(200)..link_start];
        let url = extract_href(href_area).unwrap_or_default();

        // Extract title text (between > and </a>)
        let after_marker = link_start + link_marker.len();
        let title = if let Some(gt) = html[after_marker..].find('>') {
            let text_start = after_marker + gt + 1;
            if let Some(end_tag) = html[text_start..].find("</a>") {
                strip_tags(&html[text_start..text_start + end_tag])
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Find snippet
        let snippet_marker = "class=\"result__snippet\"";
        let snippet = if let Some(snippet_start) = html[after_marker..].find(snippet_marker) {
            let snippet_start = after_marker + snippet_start + snippet_marker.len();
            if let Some(gt) = html[snippet_start..].find('>') {
                let text_start = snippet_start + gt + 1;
                // Find the closing tag
                if let Some(end) = html[text_start..].find("</") {
                    strip_tags(&html[text_start..text_start + end])
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        pos = after_marker;

        // Clean up the URL — DuckDuckGo wraps URLs in a redirect
        let clean_url = clean_ddg_url(&url);

        if !title.is_empty() || !clean_url.is_empty() {
            results.push(SearchResult {
                title: title.trim().to_string(),
                url: clean_url,
                snippet: snippet.trim().to_string(),
            });
        }
    }

    results
}

fn extract_href(html: &str) -> Option<String> {
    let href_pos = html.rfind("href=\"")?;
    let start = href_pos + 6;
    let end = html[start..].find('"')?;
    Some(html[start..start + end].to_string())
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    // Decode basic entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&nbsp;", " ")
}

fn clean_ddg_url(url: &str) -> String {
    // DuckDuckGo wraps URLs: //duckduckgo.com/l/?uddg=ENCODED_URL&...
    if let Some(uddg_pos) = url.find("uddg=") {
        let start = uddg_pos + 5;
        let end = url[start..].find('&').unwrap_or(url[start..].len());
        let encoded = &url[start..start + end];
        urldecoded(encoded)
    } else if url.starts_with("//") {
        format!("https:{url}")
    } else {
        url.to_string()
    }
}

fn urlencoded(s: &str) -> String {
    let mut result = String::new();
    for c in s.bytes() {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(c as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", c));
            }
        }
    }
    result
}

fn urldecoded(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                &s[i + 1..i + 3],
                16,
            ) {
                result.push(byte);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            result.push(b' ');
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}
