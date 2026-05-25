use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Outcome of a Ctrl+V paste attempt.
pub enum PasteOutcome {
    /// Clipboard held a bitmap; saved at `path`.
    Image { path: PathBuf },
    /// Clipboard held text.
    Text(String),
    /// Clipboard was empty or unreadable.
    Empty,
}

/// Try to pull an image first; if there isn't one, try text.
pub fn read_clipboard() -> PasteOutcome {
    let Ok(mut cb) = arboard::Clipboard::new() else { return PasteOutcome::Empty; };

    if let Ok(img) = cb.get_image() {
        match save_image_to_temp(&img) {
            Ok(path) => return PasteOutcome::Image { path },
            Err(e) => {
                tracing::warn!("failed to save pasted image: {e}");
            }
        }
    }

    if let Ok(text) = cb.get_text() {
        if !text.is_empty() {
            return PasteOutcome::Text(text);
        }
    }

    PasteOutcome::Empty
}

fn save_image_to_temp(img: &arboard::ImageData) -> Result<PathBuf, String> {
    use image::ImageEncoder;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let dir = std::env::temp_dir().join("stynx-pastes");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;
    let path = dir.join(format!("paste-{now}.png"));

    let file = std::fs::File::create(&path).map_err(|e| format!("create: {e}"))?;
    let writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::png::PngEncoder::new(writer);
    encoder
        .write_image(
            &img.bytes,
            img.width as u32,
            img.height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("encode: {e}"))?;
    Ok(path)
}
