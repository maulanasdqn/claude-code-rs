use std::sync::{Mutex, OnceLock};

static CURRENT_MODEL: OnceLock<Mutex<String>> = OnceLock::new();

fn model_store() -> &'static Mutex<String> {
    CURRENT_MODEL.get_or_init(|| Mutex::new(String::new()))
}

pub fn set_current_model(model: &str) {
    let short = model.trim_start_matches("claude-");
    if let Ok(mut g) = model_store().lock() {
        *g = short.to_string();
    }
}
