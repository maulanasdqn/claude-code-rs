use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use stynx_code_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct JwtHeader {
    alg: String,
}

pub struct JwtValidator {
    secret: Vec<u8>,
}

impl JwtValidator {
    pub fn from_env() -> AppResult<Self> {
        let secret = std::env::var("STYNX_BRIDGE_SECRET")
            .or_else(|_| std::env::var("CLAUDE_SERVER_TOKEN"))
            .map_err(|_| AppError::Provider(
                "STYNX_BRIDGE_SECRET (or legacy CLAUDE_SERVER_TOKEN) must be set for JWT signature verification".into(),
            ))?;
        if secret.len() < 32 {
            return Err(AppError::Provider(
                "STYNX_BRIDGE_SECRET must be at least 32 bytes for HS256 safety".into(),
            ));
        }
        Ok(Self { secret: secret.into_bytes() })
    }

    pub fn with_secret(secret: impl Into<Vec<u8>>) -> Self {
        Self { secret: secret.into() }
    }

    pub fn validate_token(&self, token: &str) -> AppResult<JwtClaims> {
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized);
        }

        let header_bytes = URL_SAFE_NO_PAD.decode(parts[0]).map_err(|_| AppError::Unauthorized)?;
        let header: JwtHeader = serde_json::from_slice(&header_bytes).map_err(|_| AppError::Unauthorized)?;
        if header.alg != "HS256" {
            return Err(AppError::Unauthorized);
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = {
            let mut mac = HmacSha256::new_from_slice(&self.secret)
                .map_err(|_| AppError::Unauthorized)?;
            mac.update(signing_input.as_bytes());
            mac.finalize().into_bytes()
        };

        let provided_sig = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|_| AppError::Unauthorized)?;

        if expected_sig.ct_eq(&provided_sig).unwrap_u8() != 1 {
            return Err(AppError::Unauthorized);
        }

        let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1]).map_err(|_| AppError::Unauthorized)?;
        let claims: JwtClaims = serde_json::from_slice(&payload_bytes).map_err(|_| AppError::Unauthorized)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if claims.exp <= now {
            return Err(AppError::Unauthorized);
        }
        if claims.iat > now + 60 {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }
}
