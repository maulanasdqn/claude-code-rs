use stynx_code_errors::{AppError, AppResult};

use crate::domain::Credential;
use crate::infrastructure::oauth::refresh::{
    apply_to_document, persist_to_stynx_cache, refresh_access_token, write_credentials_file,
    OAuthState, RefreshedTokens,
};

pub fn resolve_file_oauth() -> AppResult<Credential> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AppError::Provider("cannot determine home directory".to_string()))?;
    let home = std::path::PathBuf::from(home);

    let candidates = [
        home.join(".stynx").join(".credentials.json"),
        home.join(".claude").join(".credentials.json"),
    ];

    let path = candidates
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| AppError::Provider(format!(
            "no credentials file at {} or {}",
            candidates[0].display(),
            candidates[1].display(),
        )))?;

    let contents = std::fs::read_to_string(path)
        .map_err(|e| AppError::Provider(format!("cannot read {}: {e}", path.display())))?;

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

    let refresh_token = oauth
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let expires_at = oauth.get("expiresAt").and_then(|v| v.as_u64()).unwrap_or(0);

    let state = OAuthState {
        access_token,
        refresh_token,
        expires_at_ms: expires_at,
    };

    if state.needs_refresh() {
        if let Some(rt) = state.refresh_token.as_deref() {
            match refresh_access_token(rt) {
                Ok(fresh) => {
                    persist_file(path, &parsed, &fresh);
                    return Ok(Credential::ClaudeCodeOAuth {
                        access_token: fresh.access_token,
                        expires_at: fresh.expires_at_ms,
                    });
                }
                Err(e) => {
                    tracing::warn!("file OAuth token refresh failed: {e}");
                    if state.is_expired() {
                        return Err(AppError::Provider(
                            "Claude Code OAuth token expired and refresh failed. Run `claude` to refresh your session."
                                .to_string(),
                        ));
                    }
                    // Not yet expired — fall through and use the existing token.
                }
            }
        } else if state.is_expired() {
            return Err(AppError::Provider(
                "Claude Code OAuth token expired. Run `claude` to refresh your session."
                    .to_string(),
            ));
        }
    }

    Ok(Credential::ClaudeCodeOAuth {
        access_token: state.access_token,
        expires_at: state.expires_at_ms,
    })
}

/// Persist refreshed tokens back to the credentials file they came from, keeping
/// every other field intact. Falls back to Stynx's own cache on failure.
fn persist_file(path: &std::path::Path, document: &serde_json::Value, fresh: &RefreshedTokens) {
    let updated = apply_to_document(document, fresh);
    match write_credentials_file(path, &updated) {
        Ok(()) => {
            tracing::info!("refreshed Claude Code OAuth token and updated {}", path.display());
        }
        Err(e) => {
            tracing::warn!("failed to update {}: {e}", path.display());
            persist_to_stynx_cache(document, fresh);
        }
    }
}

pub fn resolve_settings_json() -> AppResult<Credential> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AppError::Provider("cannot determine home directory".to_string()))?;

    let path = std::path::PathBuf::from(home)
        .join(".stynx")
        .join("settings.json");

    let contents = std::fs::read_to_string(&path)
        .map_err(|e| AppError::Provider(format!("cannot read {}: {e}", path.display())))?;

    let parsed: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| AppError::Provider(format!("failed to parse settings.json: {e}")))?;

    let env = parsed
        .get("env")
        .ok_or_else(|| AppError::Provider("no env in settings.json".to_string()))?;

    let token = env
        .get("ANTHROPIC_AUTH_TOKEN")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Provider("no ANTHROPIC_AUTH_TOKEN in settings.json".to_string()))?
        .to_string();

    let base_url = env
        .get("ANTHROPIC_BASE_URL")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("https://api.anthropic.com")
        .to_string();

    tracing::debug!("using ANTHROPIC_AUTH_TOKEN from settings.json");

    Ok(Credential::AuthToken { token, base_url })
}
