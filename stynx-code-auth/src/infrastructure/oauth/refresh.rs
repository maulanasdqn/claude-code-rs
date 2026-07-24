use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use stynx_code_errors::{AppError, AppResult};

/// Public OAuth client id used by the Claude Code CLI. Stynx reuses the same
/// Claude Code credentials, so it must refresh against the same client id and
/// token endpoint — otherwise the rotated refresh token would not be accepted.
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

/// Refresh proactively once the access token is within this window of expiring,
/// so a token that is technically still valid at resolve time does not expire
/// mid-request and log the user out.
const REFRESH_MARGIN_MS: u64 = 5 * 60 * 1000;

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A parsed OAuth session as read from the Keychain or a credentials file.
pub struct OAuthState {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Absolute expiry in epoch milliseconds. `0` means unknown / never.
    pub expires_at_ms: u64,
}

impl OAuthState {
    /// The access token is expired or close enough to expiry that it should be
    /// refreshed now.
    pub fn needs_refresh(&self) -> bool {
        self.expires_at_ms > 0
            && now_millis().saturating_add(REFRESH_MARGIN_MS) >= self.expires_at_ms
    }

    /// The access token is already unusable.
    pub fn is_expired(&self) -> bool {
        self.expires_at_ms > 0 && now_millis() >= self.expires_at_ms
    }
}

#[derive(Debug, Clone)]
pub struct RefreshedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_ms: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

/// Exchange a refresh token for a fresh access token.
///
/// The blocking HTTP request runs on a dedicated OS thread so this is safe to
/// call from inside a Tokio runtime — `reqwest::blocking` panics if a runtime is
/// already active on the calling thread.
pub fn refresh_access_token(refresh_token: &str) -> AppResult<RefreshedTokens> {
    let refresh_owned = refresh_token.to_string();
    std::thread::spawn(move || refresh_blocking(&refresh_owned))
        .join()
        .map_err(|_| AppError::Provider("token refresh thread panicked".to_string()))?
}

fn refresh_blocking(refresh_token: &str) -> AppResult<RefreshedTokens> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Provider(format!("failed to build refresh HTTP client: {e}")))?;

    let payload = json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": CLIENT_ID,
    });

    let resp = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", "claude-cli/2.1.87 (external, cli)")
        .json(&payload)
        .send()
        .map_err(|e| AppError::Provider(format!("token refresh request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(AppError::Provider(format!(
            "token refresh rejected ({status}): {body}"
        )));
    }

    let parsed: TokenResponse = resp
        .json()
        .map_err(|e| AppError::Provider(format!("failed to parse refresh response: {e}")))?;

    let expires_in = parsed.expires_in.unwrap_or(3600);
    let expires_at_ms = now_millis().saturating_add(expires_in.saturating_mul(1000));

    Ok(RefreshedTokens {
        access_token: parsed.access_token,
        refresh_token: parsed
            .refresh_token
            .unwrap_or_else(|| refresh_token.to_string()),
        expires_at_ms,
    })
}

/// Apply refreshed tokens onto a parsed `{ "claudeAiOauth": { ... } }` document,
/// preserving every other field the store already had.
pub fn apply_to_document(document: &Value, fresh: &RefreshedTokens) -> Value {
    let mut updated = document.clone();
    let oauth = updated
        .as_object_mut()
        .and_then(|o| o.entry("claudeAiOauth").or_insert_with(|| json!({})).as_object_mut());
    if let Some(oauth) = oauth {
        oauth.insert("accessToken".into(), json!(fresh.access_token));
        oauth.insert("refreshToken".into(), json!(fresh.refresh_token));
        oauth.insert("expiresAt".into(), json!(fresh.expires_at_ms));
    }
    updated
}

/// Write a credentials document to `path` with owner-only (0600) permissions.
pub fn write_credentials_file(path: &Path, document: &Value) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Provider(format!("failed to create {}: {e}", parent.display()))
        })?;
    }
    let json = serde_json::to_string_pretty(document)
        .map_err(|e| AppError::Provider(format!("failed to serialize credentials: {e}")))?;
    std::fs::write(path, json)
        .map_err(|e| AppError::Provider(format!("failed to write {}: {e}", path.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Path to Stynx's own credentials cache (`~/.stynx/.credentials.json`).
pub fn stynx_credentials_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    Some(PathBuf::from(home).join(".stynx").join(".credentials.json"))
}

/// Best-effort fallback persistence into Stynx's own store so that, even if the
/// original store could not be updated, the next resolve still finds the fresh
/// token (via the `~/.stynx/.credentials.json` candidate) instead of logging the
/// user out.
pub fn persist_to_stynx_cache(document: &Value, fresh: &RefreshedTokens) {
    let Some(path) = stynx_credentials_path() else {
        return;
    };
    let updated = apply_to_document(document, fresh);
    if let Err(e) = write_credentials_file(&path, &updated) {
        tracing::warn!("failed to cache refreshed OAuth token to {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(expires_at_ms: u64) -> OAuthState {
        OAuthState {
            access_token: "at".to_string(),
            refresh_token: Some("rt".to_string()),
            expires_at_ms,
        }
    }

    #[test]
    fn unknown_expiry_never_refreshes() {
        let s = state(0);
        assert!(!s.needs_refresh());
        assert!(!s.is_expired());
    }

    #[test]
    fn far_future_token_is_fresh() {
        let s = state(now_millis() + 60 * 60 * 1000);
        assert!(!s.needs_refresh());
        assert!(!s.is_expired());
    }

    #[test]
    fn token_within_margin_refreshes_but_is_not_expired() {
        let s = state(now_millis() + 60 * 1000);
        assert!(s.needs_refresh());
        assert!(!s.is_expired());
    }

    #[test]
    fn past_token_is_expired_and_refreshes() {
        let s = state(now_millis().saturating_sub(1000));
        assert!(s.needs_refresh());
        assert!(s.is_expired());
    }

    #[test]
    fn apply_updates_tokens_and_preserves_other_fields() {
        let doc = json!({
            "claudeAiOauth": {
                "accessToken": "old-at",
                "refreshToken": "old-rt",
                "expiresAt": 1u64,
                "scopes": ["user:inference"],
            }
        });
        let fresh = RefreshedTokens {
            access_token: "new-at".to_string(),
            refresh_token: "new-rt".to_string(),
            expires_at_ms: 999,
        };
        let out = apply_to_document(&doc, &fresh);
        let oauth = &out["claudeAiOauth"];
        assert_eq!(oauth["accessToken"], json!("new-at"));
        assert_eq!(oauth["refreshToken"], json!("new-rt"));
        assert_eq!(oauth["expiresAt"], json!(999));
        // Unrelated fields survive the rewrite.
        assert_eq!(oauth["scopes"], json!(["user:inference"]));
    }
}
