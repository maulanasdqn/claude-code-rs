use cc_errors::{AppError, AppResult};

#[derive(Debug, Clone)]
pub enum Credential {
    ApiKey {
        api_key: String,
        base_url: String,
    },
    ClaudeCodeOAuth {
        access_token: String,
        expires_at: u64,
    },
}

impl Credential {
    pub fn base_url(&self) -> &str {
        match self {
            Credential::ApiKey { base_url, .. } => base_url,
            Credential::ClaudeCodeOAuth { .. } => "https://api.anthropic.com",
        }
    }

    pub fn is_oauth(&self) -> bool {
        matches!(self, Credential::ClaudeCodeOAuth { .. })
    }
}

pub fn resolve_credential() -> AppResult<Credential> {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        tracing::info!("using Anthropic API key");
        return Ok(Credential::ApiKey {
            api_key: key,
            base_url: "https://api.anthropic.com".to_string(),
        });
    }

    match resolve_keychain_oauth() {
        Ok(cred) => {
            tracing::info!("using Claude Code OAuth from macOS Keychain");
            return Ok(cred);
        }
        Err(e) => {
            tracing::debug!("keychain OAuth not available: {e}");
        }
    }

    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        tracing::info!("using OpenRouter");
        return Ok(Credential::ApiKey {
            api_key: key,
            base_url: "https://openrouter.ai/api".to_string(),
        });
    }

    Err(AppError::Provider(
        "no credentials found. Set ANTHROPIC_API_KEY, OPENROUTER_API_KEY, or log in with `claude`"
            .to_string(),
    ))
}

fn resolve_keychain_oauth() -> AppResult<Credential> {
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
