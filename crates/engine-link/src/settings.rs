// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! App settings kept in the Capsule until the engine stores settings.
//!
//! One JSON object per app, at `settings/apps/<app id>.json`. Everything is
//! read once at launch, before the event loop. After that, reads come from
//! memory and each change is written by a background thread, so an app
//! changing a setting never waits on the disk. A write goes to a temporary
//! file that is then renamed over the old one, so a crash leaves the old
//! settings or the new ones, never half of either.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

/// Where app settings live, inside the Capsule.
pub const APPS_DIR: &str = "settings/apps";

type Values = HashMap<String, Map<String, Value>>;

/// Settings for every app, persisted in the Capsule.
pub struct FileSettings {
    values: Arc<Mutex<Values>>,
    writes: Mutex<mpsc::Sender<String>>,
}

/// An app id that is safe as a file name: lowercase letters, digits, dots
/// and dashes, as in `com.isonomic.chat`.
fn file_name(app: &str) -> Option<String> {
    let safe = !app.is_empty()
        && !app.starts_with('.')
        && app
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-');
    safe.then(|| format!("{app}.json"))
}

fn load(dir: &Path) -> io::Result<Values> {
    let mut values = Values::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(values),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let path = entry?.path();
        let Some(app) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(app) = app.strip_suffix(".json") else {
            continue;
        };
        if file_name(app).is_none() {
            continue;
        }
        // A file that does not parse is left alone and read as empty, so
        // one bad file never stops the window.
        let text = std::fs::read_to_string(&path)?;
        if let Ok(Value::Object(map)) = serde_json::from_str(&text) {
            values.insert(app.to_string(), map);
        }
    }
    Ok(values)
}

fn write(dir: &Path, app: &str, map: &Map<String, Value>) -> io::Result<()> {
    let Some(name) = file_name(app) else {
        return Ok(());
    };
    std::fs::create_dir_all(dir)?;
    let target = dir.join(&name);
    let temporary = dir.join(format!(".{name}.tmp"));
    let text = serde_json::to_string_pretty(map).map_err(io::Error::other)?;
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, &target)
}

impl FileSettings {
    /// Read every app's settings from the Capsule at `root`, and start the
    /// writer. Blocks on the disk: call it before the event loop.
    pub fn open(root: &Path) -> io::Result<Self> {
        let dir: PathBuf = root.join(APPS_DIR);
        let values = Arc::new(Mutex::new(load(&dir)?));
        let (writes, pending) = mpsc::channel::<String>();
        let shared = Arc::clone(&values);
        std::thread::Builder::new()
            .name("app-settings".into())
            .spawn(move || {
                while let Ok(app) = pending.recv() {
                    let map = shared.lock().unwrap().get(&app).cloned();
                    let map = map.unwrap_or_default();
                    if let Err(error) = write(&dir, &app, &map) {
                        eprintln!("[isonomic-desktop] app settings for {app} not saved: {error}");
                    }
                }
            })?;
        Ok(Self {
            values,
            writes: Mutex::new(writes),
        })
    }

    pub fn get(&self, app: &str, key: &str) -> Option<Value> {
        self.values.lock().unwrap().get(app)?.get(key).cloned()
    }

    pub fn set(&self, app: &str, key: &str, value: Value) {
        let mut values = self.values.lock().unwrap();
        values
            .entry(app.to_string())
            .or_default()
            .insert(key.to_string(), value);
        drop(values);
        self.save(app);
    }

    pub fn remove(&self, app: &str, key: &str) {
        let removed = match self.values.lock().unwrap().get_mut(app) {
            Some(map) => map.remove(key).is_some(),
            None => false,
        };
        if removed {
            self.save(app);
        }
    }

    fn save(&self, app: &str) {
        let _ = self.writes.lock().unwrap().send(app.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Wait for the writer: reopen until the file says what was set.
    fn reopened_until(root: &Path, done: impl Fn(&FileSettings) -> bool) -> FileSettings {
        for _ in 0..200 {
            let settings = FileSettings::open(root).unwrap();
            if done(&settings) {
                return settings;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the settings never reached the disk");
    }

    #[test]
    fn settings_survive_a_reopen_and_stay_per_app() {
        let capsule = tempfile::tempdir().unwrap();
        let settings = FileSettings::open(capsule.path()).unwrap();
        settings.set("com.isonomic.chat", "model", json!("chat-small"));
        settings.set("com.isonomic.notes", "model", json!("other"));
        assert_eq!(
            settings.get("com.isonomic.chat", "model"),
            Some(json!("chat-small"))
        );

        let again = reopened_until(capsule.path(), |s| {
            s.get("com.isonomic.notes", "model").is_some()
        });
        assert_eq!(
            again.get("com.isonomic.chat", "model"),
            Some(json!("chat-small"))
        );
        assert_eq!(
            again.get("com.isonomic.notes", "model"),
            Some(json!("other"))
        );

        again.remove("com.isonomic.chat", "model");
        let after = reopened_until(capsule.path(), |s| {
            s.get("com.isonomic.chat", "model").is_none()
        });
        assert_eq!(
            after.get("com.isonomic.notes", "model"),
            Some(json!("other"))
        );
    }

    #[test]
    fn unsafe_ids_and_bad_files_are_ignored() {
        assert_eq!(
            file_name("com.isonomic.chat").as_deref(),
            Some("com.isonomic.chat.json")
        );
        for bad in ["", "../x", "a/b", ".hidden", "Upper"] {
            assert_eq!(file_name(bad), None, "{bad}");
        }
        let capsule = tempfile::tempdir().unwrap();
        let dir = capsule.path().join(APPS_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("com.isonomic.chat.json"), "not json").unwrap();
        let settings = FileSettings::open(capsule.path()).unwrap();
        assert_eq!(settings.get("com.isonomic.chat", "model"), None);
    }
}
