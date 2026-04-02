use claude_rust_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub token_type: String,
}

pub trait TokenStore {
    fn save(&self, tokens: &OAuthTokens) -> AppResult<()>;
    fn load(&self) -> Option<OAuthTokens>;
    fn clear(&self) -> AppResult<()>;
}

pub struct FileTokenStore {
    path: std::path::PathBuf,
}

impl FileTokenStore {
    pub fn new() -> AppResult<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| AppError::Provider("cannot determine home directory".to_string()))?;

        let path = std::path::PathBuf::from(home)
            .join(".claude")
            .join(".credentials.json");

        Ok(Self { path })
    }
}

impl TokenStore for FileTokenStore {
    fn save(&self, tokens: &OAuthTokens) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Provider(format!(
                    "failed to create credentials directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let json = serde_json::to_string_pretty(tokens)
            .map_err(|e| AppError::Provider(format!("failed to serialize tokens: {e}")))?;

        std::fs::write(&self.path, json).map_err(|e| {
            AppError::Provider(format!(
                "failed to write credentials to {}: {e}",
                self.path.display()
            ))
        })?;

        Ok(())
    }

    fn load(&self) -> Option<OAuthTokens> {
        let contents = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    fn clear(&self) -> AppResult<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path).map_err(|e| {
                AppError::Provider(format!(
                    "failed to remove credentials file {}: {e}",
                    self.path.display()
                ))
            })?;
        }
        Ok(())
    }
}
