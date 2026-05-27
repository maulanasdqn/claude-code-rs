use regex::Regex;
use std::sync::LazyLock;

static FILE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)@([\w./\-~]+)").expect("invalid regex"));

const MAX_FILE_SIZE: u64 = 100 * 1024;

const MAX_IMAGE_SIZE: u64 = 5 * 1024 * 1024;

static IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

pub fn expand_file_references(input: &str) -> String {
    let mut file_blocks = Vec::new();

    for cap in FILE_REF_RE.captures_iter(input) {
        let path_str = match cap.get(1) {
            Some(m) => m.as_str(),
            None => continue,
        };

        let path = std::path::Path::new(path_str);

        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() || metadata.len() > MAX_FILE_SIZE {
            continue;
        }

        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        file_blocks.push(format!(
            "<file path=\"{path_str}\">{contents}</file>"
        ));
    }

    if file_blocks.is_empty() {
        return input.to_string();
    }

    let mut result = input.to_string();
    for block in file_blocks {
        result.push_str("\n\n");
        result.push_str(&block);
    }
    result
}

pub fn expand_message_content(input: &str) -> Vec<stynx_code_types::ContentBlock> {
    use base64::Engine;

    let mut text = input.to_string();
    let mut text_file_blocks: Vec<String> = Vec::new();
    let mut image_blocks: Vec<stynx_code_types::ContentBlock> = Vec::new();

    for cap in FILE_REF_RE.captures_iter(input) {
        let path_str = match cap.get(1) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let path = std::path::Path::new(path_str);
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !metadata.is_file() { continue; }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if IMAGE_EXTS.contains(&ext.as_str()) {
            if metadata.len() > MAX_IMAGE_SIZE { continue; }
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let media_type = match ext.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "gif"  => "image/gif",
                "webp" => "image/webp",
                _      => "image/png",
            };
            let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
            image_blocks.push(stynx_code_types::ContentBlock::Image {
                media_type: media_type.to_string(),
                data: encoded,
            });
        } else {
            if metadata.len() > MAX_FILE_SIZE { continue; }
            let contents = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            text_file_blocks.push(format!("<file path=\"{path_str}\">{contents}</file>"));
        }
    }

    for block in text_file_blocks {
        text.push_str("\n\n");
        text.push_str(&block);
    }

    let mut result: Vec<stynx_code_types::ContentBlock> =
        vec![stynx_code_types::ContentBlock::Text { text }];
    result.extend(image_blocks);
    result
}
