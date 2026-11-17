// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the window's notice works with: the kept reports and the setting,
//! behind one handle, so the shell never holds a path. Every call touches
//! the disk; call them off the foreground thread.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::settings::{self, ReportSettings};
use crate::store::{Pending, Store};

pub struct Kept {
    capsule: PathBuf,
    store: Store,
}

/// The Capsule's id from its `capsule.json`, or empty when unreadable.
fn capsule_id(capsule: &Path) -> String {
    std::fs::read_to_string(capsule.join("capsule.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["capsule_id"].as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Kept {
    pub fn in_capsule(capsule: &Path) -> Self {
        Self {
            capsule: capsule.to_path_buf(),
            store: Store::in_capsule(capsule),
        }
    }

    pub fn pending(&self) -> Vec<Pending> {
        self.store.pending()
    }

    pub fn discard(&self, report: &Pending) -> std::io::Result<()> {
        self.store.discard(report)
    }

    pub fn settings(&self) -> ReportSettings {
        settings::load(&self.capsule)
    }

    pub fn save_settings(&self, settings: &ReportSettings) -> std::io::Result<()> {
        settings::save(&self.capsule, settings)
    }

    /// `report`'s id: its own `id` when it has one (every report written
    /// since format 1 carried one), else [`crate::send::derived_id`].
    pub fn report_id(&self, report: &Pending) -> String {
        match report.document.get("id").and_then(Value::as_str) {
            Some(id) if isonomic_reports::id::is_id(id) => id.to_string(),
            _ => {
                let name = report
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                crate::send::derived_id(&capsule_id(&self.capsule), name)
            }
        }
    }
}

/// Exactly the bytes that would be sent for `report`: the kept document
/// with its `id` (when there is one to send), and, only when the person
/// ticked the box, their Isonomic ID as `isonomic_id`. Neither is ever in a
/// kept file, and neither is scrubbed: both are added after.
pub fn outgoing(report: &Pending, report_id: Option<&str>, isonomic_id: Option<&str>) -> String {
    if report_id.is_none() && isonomic_id.is_none() {
        return report.bytes.clone();
    }
    let mut document = report.document.clone();
    if let Value::Object(fields) = &mut document {
        if let Some(id) = report_id {
            fields.insert("id".into(), Value::String(id.to_string()));
        }
        if let Some(id) = isonomic_id {
            fields.insert("isonomic_id".into(), Value::String(id.to_string()));
        }
    }
    serde_json::to_string_pretty(&document).unwrap_or_else(|_| report.bytes.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_bytes_shown_are_the_file_and_the_id_only_when_ticked() {
        let capsule = tempfile::tempdir().unwrap();
        let kept = Kept::in_capsule(capsule.path());
        let store = Store::in_capsule(capsule.path());
        store
            .save(&json!({ "format": 1, "kind": "panic" }), 1)
            .unwrap();
        let report = kept.pending().remove(0);
        assert_eq!(outgoing(&report, None, None), report.bytes);
        assert!(!report.bytes.contains("isonomic_id"));
        let with_id: Value =
            serde_json::from_str(&outgoing(&report, Some("r1"), Some("ISO-1"))).unwrap();
        assert_eq!(with_id["isonomic_id"], "ISO-1");
        assert_eq!(with_id["id"], "r1");
        assert_eq!(with_id["kind"], "panic");
        let plain: Value = serde_json::from_str(&outgoing(&report, Some("r1"), None)).unwrap();
        assert!(plain.get("isonomic_id").is_none());
        kept.discard(&report).unwrap();
        assert!(kept.pending().is_empty());
    }

    #[test]
    fn a_report_s_own_id_is_used_and_an_older_one_gets_a_derived_one() {
        let capsule = tempfile::tempdir().unwrap();
        std::fs::write(
            capsule.path().join("capsule.json"),
            r#"{ "format": 1, "capsule_id": "cap-1" }"#,
        )
        .unwrap();
        let kept = Kept::in_capsule(capsule.path());
        let store = Store::in_capsule(capsule.path());
        let own = "0123456789abcdef0123456789abcdef";
        store
            .save(&json!({ "format": 1, "kind": "panic", "id": own }), 1)
            .unwrap();
        store
            .save(&json!({ "format": 1, "kind": "hang" }), 2)
            .unwrap();
        let pending = kept.pending();
        assert_eq!(kept.report_id(&pending[0]), own);
        let derived = kept.report_id(&pending[1]);
        assert!(isonomic_reports::id::is_id(&derived), "{derived}");
        assert_eq!(
            derived,
            crate::send::derived_id("cap-1", "0000000000000002-hang")
        );
    }
}
