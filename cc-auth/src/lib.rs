use cc_errors::{AppError, AppResult};
use std::process::Command;

const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
const OAUTH_BASE_URL: &str = "https://api.claude.ai";

#[derive(Debug, Clone)]
pub enum Credential {
    /// Raw API key (sk-ant-api...) against api.anthropic.com
    ApiKey(String),
    /// OAuth access token (sk-ant-oat...) against api.claude.ai
    OAuthToken {
        access_token: String,
        base_url: String,
    },
}

impl Credential {
    pub fn auth_header_value(&self) -> String {
        match self {
            Credential::ApiKey(key) => key.clone(),
            Credential::OAuthToken { access_token, .. } => access_token.clone(),
        }
    }

    pub fn base_url(&self) -> &str {
        match self {
            Credential::ApiKey(_) => "https://api.anthropic.com",
            Credential::OAuthToken { base_url, .. } => base_url,
        }
    }

    pub fn is_oauth(&self) -> bool {
        matches!(self, Credential::OAuthToken { .. })
    }
}

/// Resolve credentials in priority order:
/// 1. ANTHROPIC_API_KEY env var
/// 2. macOS Keychain (Claude Code OAuth token)
pub fn resolve_credential() -> AppResult<Credential> {
    // 1. Check env var
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        tracing::info!("using ANTHROPIC_API_KEY from environment");
        return Ok(Credential::ApiKey(key));
    }

    // 2. Try macOS Keychain
    tracing::info!("no ANTHROPIC_API_KEY set, trying macOS Keychain");
    match read_keychain_credential() {
        Ok(cred) => {
            tracing::info!("using OAuth token from macOS Keychain");
            Ok(cred)
        }
        Err(e) => Err(AppError::Provider(format!(
            "no credentials found. Set ANTHROPIC_API_KEY or log in with Claude Code first. \
             Keychain error: {e}"
        ))),
    }
}

fn read_keychain_credential() -> AppResult<Credential> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()
        .map_err(|e| AppError::Provider(format!("failed to run `security`: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Provider(format!(
            "keychain lookup failed: {stderr}"
        )));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let json: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| AppError::Provider(format!("failed to parse keychain JSON: {e}")))?;

    let access_token = json
        .get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("no accessToken in keychain data".into()))?
        .to_string();

    Ok(Credential::OAuthToken {
        access_token,
        base_url: OAUTH_BASE_URL.to_string(),
    })
}
