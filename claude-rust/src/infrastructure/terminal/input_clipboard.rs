pub(super) fn read_clipboard_image() -> Option<(String, String)> {
    use base64::Engine;
    let tmp = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let tmp_path = format!("{}/claude-rust-paste-{}.png", tmp.trim_end_matches('/'), std::process::id());

    if let Ok(out) = std::process::Command::new("wl-paste")
        .args(["--type", "image/png", "--no-newline"])
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success() && !out.stdout.is_empty() {
            return Some(("image/png".to_string(), base64::engine::general_purpose::STANDARD.encode(&out.stdout)));
        }

    if let Ok(out) = std::process::Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "image/png", "-o"])
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success() && !out.stdout.is_empty() {
            return Some(("image/png".to_string(), base64::engine::general_purpose::STANDARD.encode(&out.stdout)));
        }

    let script = format!(
        "try\n\
         set imgData to (the clipboard as «class PNGf»)\n\
         set fileRef to open for access POSIX file \"{tmp_path}\" with write permission\n\
         set eof fileRef to 0\n\
         write imgData to fileRef\n\
         close access fileRef\n\
         return \"ok\"\n\
         on error\n\
         return \"err\"\n\
         end try"
    );
    if let Ok(out) = std::process::Command::new("osascript")
        .arg("-e").arg(&script)
        .stderr(std::process::Stdio::null())
        .output()
        && String::from_utf8_lossy(&out.stdout).trim() == "ok"
            && let Ok(data) = std::fs::read(&tmp_path) {
                let _ = std::fs::remove_file(&tmp_path);
                if !data.is_empty() {
                    return Some(("image/png".to_string(), base64::engine::general_purpose::STANDARD.encode(&data)));
                }
            }

    if let Ok(out) = std::process::Command::new("pngpaste")
        .arg(&tmp_path)
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success()
            && let Ok(data) = std::fs::read(&tmp_path) {
                let _ = std::fs::remove_file(&tmp_path);
                if !data.is_empty() {
                    return Some(("image/png".to_string(), base64::engine::general_purpose::STANDARD.encode(&data)));
                }
            }

    None
}
