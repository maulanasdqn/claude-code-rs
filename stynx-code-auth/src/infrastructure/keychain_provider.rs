use stynx_code_errors::{AppError, AppResult};

use crate::domain::Credential;
use crate::infrastructure::oauth::refresh::{
    apply_to_document, now_millis, persist_to_stynx_cache, refresh_access_token, OAuthState,
    RefreshedTokens,
};

const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

pub fn resolve_keychain_oauth() -> AppResult<Credential> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
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
                    persist_keychain(&parsed, &fresh);
                    return Ok(Credential::ClaudeCodeOAuth {
                        access_token: fresh.access_token,
                        expires_at: fresh.expires_at_ms,
                    });
                }
                Err(e) => {
                    tracing::warn!("Keychain OAuth token refresh failed: {e}");
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

/// Write the refreshed tokens back to the Keychain so Stynx and the real
/// `claude` CLI keep sharing the same rotated credentials. Falls back to
/// Stynx's own credentials cache if the Keychain update fails.
fn persist_keychain(document: &serde_json::Value, fresh: &RefreshedTokens) {
    let updated = apply_to_document(document, fresh);
    let serialized = match serde_json::to_string(&updated) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("failed to serialize refreshed Keychain credentials: {e}");
            persist_to_stynx_cache(document, fresh);
            return;
        }
    };

    let account = keychain_account();

    let mut cmd = std::process::Command::new("security");
    cmd.args(["add-generic-password", "-U", "-s", KEYCHAIN_SERVICE]);
    if let Some(acct) = account.as_deref().filter(|a| !a.is_empty()) {
        cmd.args(["-a", acct]);
    }
    cmd.args(["-w", &serialized]);

    match cmd.output() {
        Ok(o) if o.status.success() => {
            tracing::info!(
                expires_in_s = (fresh.expires_at_ms.saturating_sub(now_millis())) / 1000,
                "refreshed Claude Code OAuth token and updated Keychain"
            );
        }
        Ok(o) => {
            tracing::warn!(
                "failed to update Keychain ({}): {}",
                o.status,
                String::from_utf8_lossy(&o.stderr).trim()
            );
            persist_to_stynx_cache(document, fresh);
        }
        Err(e) => {
            tracing::warn!("failed to run `security` to update Keychain: {e}");
            persist_to_stynx_cache(document, fresh);
        }
    }
}

/// Read the account (`acct`) attribute of the existing Keychain item so the
/// update targets the same entry instead of creating a duplicate.
fn keychain_account() -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("\"acct\"") {
            let value = rest.rsplit_once('=').map(|(_, v)| v.trim().trim_matches('"'));
            if let Some(value) = value
                && !value.is_empty()
                && value != "<NULL>"
            {
                return Some(value.to_string());
            }
        }
    }
    None
}
