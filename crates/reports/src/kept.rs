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
}

/// Exactly the bytes that would be sent for `report`: the file as kept, or,
/// when the person ticked the box, the same document with their Isonomic
/// ID added. The ID is never in a kept file.
pub fn outgoing(report: &Pending, isonomic_id: Option<&str>) -> String {
    let Some(id) = isonomic_id else {
        return report.bytes.clone();
    };
    let mut document = report.document.clone();
    if let Value::Object(fields) = &mut document {
        fields.insert("isonomic_id".into(), Value::String(id.to_string()));
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
        assert_eq!(outgoing(&report, None), report.bytes);
        assert!(!report.bytes.contains("isonomic_id"));
        let with_id: Value = serde_json::from_str(&outgoing(&report, Some("ISO-1"))).unwrap();
        assert_eq!(with_id["isonomic_id"], "ISO-1");
        assert_eq!(with_id["kind"], "panic");
        kept.discard(&report).unwrap();
        assert!(kept.pending().is_empty());
    }
}
