use std::fmt;

#[derive(Clone)]
pub enum Credential {
    ApiKey {
        api_key: String,
        base_url: String,
    },
    AuthToken {
        token: String,
        base_url: String,
    },
    ClaudeCodeOAuth {
        access_token: String,
        expires_at: u64,
    },
}

impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credential::ApiKey { base_url, .. } => f
                .debug_struct("ApiKey")
                .field("api_key", &"[redacted]")
                .field("base_url", base_url)
                .finish(),
            Credential::AuthToken { base_url, .. } => f
                .debug_struct("AuthToken")
                .field("token", &"[redacted]")
                .field("base_url", base_url)
                .finish(),
            Credential::ClaudeCodeOAuth { expires_at, .. } => f
                .debug_struct("ClaudeCodeOAuth")
                .field("access_token", &"[redacted]")
                .field("expires_at", expires_at)
                .finish(),
        }
    }
}

impl Credential {
    pub fn base_url(&self) -> &str {
        match self {
            Credential::ApiKey { base_url, .. } => base_url,
            Credential::AuthToken { base_url, .. } => base_url,
            Credential::ClaudeCodeOAuth { .. } => "https://api.anthropic.com",
        }
    }

    pub fn is_oauth(&self) -> bool {
        matches!(
            self,
            Credential::ClaudeCodeOAuth { .. } | Credential::AuthToken { .. }
        )
    }
}
