use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use claude_rust_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

pub struct JwtValidator;

impl JwtValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_token(&self, token: &str) -> AppResult<JwtClaims> {
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized);
        }

        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| AppError::Unauthorized)?;

        let claims: JwtClaims = serde_json::from_slice(&payload_bytes)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(claims)
    }
}

impl Default for JwtValidator {
    fn default() -> Self {
        Self::new()
    }
}
