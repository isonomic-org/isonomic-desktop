// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reports waiting on this Mac: `reports/pending/` in the Capsule, one file
//! each. Deleted when the person says Don't send, after [`KEEP_DAYS`], or
//! beyond [`KEEP_MOST`], oldest first. Nothing here sends.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// Unsent reports are kept this long.
pub const KEEP_DAYS: u64 = 30;
/// At most this many unsent reports are kept.
pub const KEEP_MOST: usize = 20;

/// One kept report.
#[derive(Debug, Clone, PartialEq)]
pub struct Pending {
    pub path: PathBuf,
    /// The exact bytes that would be sent.
    pub bytes: String,
    pub document: Value,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// The store inside the Capsule at `capsule`.
    pub fn in_capsule(capsule: &Path) -> Self {
        Self {
            dir: capsule.join("reports").join("pending"),
        }
    }

    /// Write `document` as a new report, named so files sort oldest first.
    pub fn save(&self, document: &Value, unix_millis: u128) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.dir)?;
        let kind = document["kind"].as_str().unwrap_or("report");
        let mut path = self.dir.join(format!("{unix_millis:016}-{kind}.json"));
        let mut n = 1;
        while path.exists() {
            path = self.dir.join(format!("{unix_millis:016}-{kind}-{n}.json"));
            n += 1;
        }
        let bytes = serde_json::to_string_pretty(document).map_err(std::io::Error::other)?;
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    /// Every kept report, oldest first. A file that doesn't read is skipped.
    pub fn pending(&self) -> Vec<Pending> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        paths
            .into_iter()
            .filter_map(|path| {
                let bytes = std::fs::read_to_string(&path).ok()?;
                let document = serde_json::from_str(&bytes).ok()?;
                Some(Pending {
                    path,
                    bytes,
                    document,
                })
            })
            .collect()
    }

    /// Don't send: the report is deleted.
    pub fn discard(&self, report: &Pending) -> std::io::Result<()> {
        match std::fs::remove_file(&report.path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    /// Delete what is past [`KEEP_DAYS`] old, then the oldest beyond
    /// [`KEEP_MOST`]. Ages come from the file names.
    pub fn prune(&self, now_unix_millis: u128) {
        let max_age = u128::from(KEEP_DAYS) * 24 * 3600 * 1000;
        let mut kept = Vec::new();
        for report in self.pending() {
            let born = report
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.split('-').next())
                .and_then(|n| n.parse::<u128>().ok())
                .unwrap_or(now_unix_millis);
            if now_unix_millis.saturating_sub(born) > max_age {
                let _ = self.discard(&report);
            } else {
                kept.push(report);
            }
        }
        let excess = kept.len().saturating_sub(KEEP_MOST);
        for report in kept.iter().take(excess) {
            let _ = self.discard(report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const DAY: u128 = 24 * 3600 * 1000;

    #[test]
    fn reports_are_kept_shown_exactly_and_discarded() {
        let capsule = tempfile::tempdir().unwrap();
        let store = Store::in_capsule(capsule.path());
        assert!(store.pending().is_empty());
        store
            .save(&json!({"kind": "panic", "message": "x"}), 1_000)
            .unwrap();
        store.save(&json!({"kind": "hang"}), 2_000).unwrap();
        let pending = store.pending();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].document["kind"], "panic");
        assert_eq!(
            serde_json::from_str::<Value>(&pending[0].bytes).unwrap(),
            pending[0].document
        );
        store.discard(&pending[0]).unwrap();
        assert_eq!(store.pending().len(), 1);
    }

    #[test]
    fn old_and_excess_reports_are_pruned() {
        let capsule = tempfile::tempdir().unwrap();
        let store = Store::in_capsule(capsule.path());
        let now = 100 * DAY;
        store.save(&json!({"kind": "old"}), now - 31 * DAY).unwrap();
        for i in 0..(KEEP_MOST + 3) {
            store
                .save(&json!({"kind": "panic"}), now - DAY + i as u128)
                .unwrap();
        }
        store.prune(now);
        let pending = store.pending();
        assert_eq!(pending.len(), KEEP_MOST);
        assert!(pending.iter().all(|p| p.document["kind"] == "panic"));
    }
}
