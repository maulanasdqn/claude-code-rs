use claude_rust_errors::{AppError, AppResult};

use crate::domain::Credential;

pub fn resolve_keychain_oauth() -> AppResult<Credential> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| AppError::Provider(format!("failed to run `security`: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Provider(
            "no Claude Code credentials in Keychain".to_string(),
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Provider(format!("failed to parse Keychain JSON: {e}")))?;

    let oauth = parsed
        .get("claudeAiOauth")
        .ok_or_else(|| AppError::Provider("no claudeAiOauth in Keychain data".to_string()))?;

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
