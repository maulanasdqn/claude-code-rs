use std::collections::HashMap;
use std::path::PathBuf;

fn keys_file() -> Option<PathBuf> {
    stynx_code_config::home_dir().map(|home| home.join(".stynx").join("keys.json"))
}

fn read_keys() -> HashMap<String, String> {
    keys_file()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn apply_persisted_keys() {
    for (env_name, value) in read_keys() {
        let already_set = std::env::var(&env_name)
            .ok()
            .filter(|existing| !existing.trim().is_empty())
            .is_some();
        if !already_set {
            unsafe { std::env::set_var(&env_name, &value) };
        }
    }
}

pub fn save_provider_key(env_name: &str, value: &str) -> Result<(), String> {
    let path = keys_file().ok_or("cannot determine home directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut keys = read_keys();
    keys.insert(env_name.to_string(), value.to_string());
    let body = serde_json::to_string_pretty(&keys).map_err(|error| error.to_string())?;
    std::fs::write(&path, body).map_err(|error| error.to_string())?;
    unsafe { std::env::set_var(env_name, value) };
    Ok(())
}
