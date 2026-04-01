use claude_rust_errors::{AppError, AppResult};

use crate::domain::Credential;

/// Reads OAuth credentials from `~/.claude/.credentials.json`,
/// which is where the Claude Code CLI stores them on Linux (and as a fallback on all platforms).
pub fn resolve_file_oauth() -> AppResult<Credential> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AppError::Provider("cannot determine home directory".to_string()))?;

    let path = std::path::PathBuf::from(home)
        .join(".claude")
        .join(".credentials.json");

    let contents = std::fs::read_to_string(&path).map_err(|e| {
        AppError::Provider(format!(
            "cannot read {}: {e}",
            path.display()
        ))
    })?;

    let parsed: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| AppError::Provider(format!("failed to parse credentials JSON: {e}")))?;

    let oauth = parsed
        .get("claudeAiOauth")
        .ok_or_else(|| AppError::Provider("no claudeAiOauth in credentials file".to_string()))?;

    let access_token = oauth
        .get("accessToken")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("no accessToken in OAuth data".to_string()))?
        .to_string();

    let expires_at = oauth
        .get("expiresAt")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if expires_at > 0 {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms > expires_at {
            return Err(AppError::Provider(
                "Claude Code OAuth token expired. Run `claude` to refresh your session."
                    .to_string(),
            ));
        }
    }

    Ok(Credential::ClaudeCodeOAuth {
        access_token,
        expires_at,
    })
}
