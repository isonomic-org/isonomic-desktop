//! The window's last size, kept in the Capsule's `settings/window.json`, so
//! it opens as the person left it. Read once before the window opens;
//! written off the foreground thread whenever the size changes.

use std::path::{Path, PathBuf};

use serde_json::json;
use ui::{px, size, Pixels, Size};

fn file(root: &Path) -> PathBuf {
    root.join("settings").join("window.json")
}

/// The size last kept, if any reads back as a usable size.
pub fn load(root: &Path) -> Option<Size<Pixels>> {
    let text = std::fs::read_to_string(file(root)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let (w, h) = (value["width"].as_f64()?, value["height"].as_f64()?);
    (w >= 200.0 && h >= 200.0).then(|| size(px(w as f32), px(h as f32)))
}

/// Keep `kept` as the last size. A write that fails is only logged.
pub fn save(root: &Path, kept: Size<Pixels>) {
    let path = file(root);
    let value = json!({ "width": f32::from(kept.width), "height": f32::from(kept.height) });
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::write(&path, value.to_string()));
    if let Err(error) = written {
        reports::log::line(&format!("window size not kept: {}", error.kind()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_is_kept_and_read_back_and_nonsense_is_not() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(load(root.path()), None);
        save(root.path(), size(px(1280.), px(800.)));
        assert_eq!(load(root.path()), Some(size(px(1280.), px(800.))));
        std::fs::write(file(root.path()), r#"{"width": 10, "height": 10}"#).unwrap();
        assert_eq!(load(root.path()), None);
    }
}
