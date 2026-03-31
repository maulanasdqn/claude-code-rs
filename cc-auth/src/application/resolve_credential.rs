use cc_errors::{AppError, AppResult};

use crate::domain::Credential;
use crate::infrastructure::resolve_keychain_oauth;

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
