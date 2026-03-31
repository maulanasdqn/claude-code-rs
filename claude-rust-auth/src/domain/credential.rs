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
