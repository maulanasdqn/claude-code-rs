use serde::Serialize;

const MAX_FILE_BYTES: usize = 512 * 1024;
const MAX_FETCH_BYTES: usize = 2 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedReference {
    pub content_type: String,
    pub body: String,
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    let bytes = tokio::fs::read(&path).await.map_err(|error| error.to_string())?;
    let capped = &bytes[..bytes.len().min(MAX_FILE_BYTES)];
    Ok(String::from_utf8_lossy(capped).to_string())
}

const MAX_INDEX_ENTRIES: usize = 3000;
const IGNORED_DIRS: [&str; 8] =
    ["node_modules", "target", ".git", "build", "DerivedData", "dist", ".next", "gen"];

#[tauri::command]
pub async fn list_project_files(root: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let base = std::path::PathBuf::from(&root);
        let mut out = Vec::new();
        walk(&base, &base, &mut out);
        out
    })
    .await
    .map_err(|error| error.to_string())
}

fn walk(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    if out.len() >= MAX_INDEX_ENTRIES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if out.len() >= MAX_INDEX_ENTRIES {
            return;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }
        if path.is_dir() {
            walk(base, &path, out);
        } else if let Ok(relative) = path.strip_prefix(base) {
            out.push(relative.to_string_lossy().to_string());
        }
    }
}

#[tauri::command]
pub async fn fetch_reference(url: String) -> Result<FetchedReference, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("only http(s) URLs are supported".to_string());
    }
    let response = reqwest::get(&url).await.map_err(|error| error.to_string())?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let capped = &bytes[..bytes.len().min(MAX_FETCH_BYTES)];
    Ok(FetchedReference {
        content_type,
        body: String::from_utf8_lossy(capped).to_string(),
    })
}
