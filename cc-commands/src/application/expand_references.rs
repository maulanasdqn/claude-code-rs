use regex::Regex;
use std::sync::LazyLock;

static FILE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)@([\w./\-~]+)").expect("invalid regex"));

const MAX_FILE_SIZE: u64 = 100 * 1024;

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
