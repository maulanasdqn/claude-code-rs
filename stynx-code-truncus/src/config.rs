use serde::Deserialize;
use std::path::PathBuf;

/// Truncus client config. Resolved from `TRUNCUS_URL` + `TRUNCUS_TOKEN` env vars,
/// falling back to `~/.config/truncus/config.toml` — the same file the `truncus`
/// CLI writes, so a single `truncus install` configures both tools.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub url: String,
    pub token: String,
}

impl Config {
    pub fn load() -> Option<Config> {
        if let (Ok(url), Ok(token)) = (std::env::var("TRUNCUS_URL"), std::env::var("TRUNCUS_TOKEN"))
            && !url.trim().is_empty()
            && !token.trim().is_empty()
        {
            return Some(Config { url, token });
        }
        let raw = std::fs::read_to_string(Self::path()?).ok()?;
        let cfg: Config = toml::from_str(&raw).ok()?;
        if cfg.url.trim().is_empty() || cfg.token.trim().is_empty() {
            return None;
        }
        Some(cfg)
    }

    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("truncus").join("config.toml"))
    }
}
