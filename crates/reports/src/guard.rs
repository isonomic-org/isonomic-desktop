// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The crash-loop guard. Each launch counts itself in the Capsule's
//! `reports/launches` before the window opens; surviving [`SURVIVED_SECS`]
//! or quitting cleanly resets the count. A window that has failed to get
//! that far [`LIMIT`] times running starts in a safe state: no app is
//! mounted, and Home says why.

use std::path::{Path, PathBuf};

/// Launches that ended early, in a row, before the window starts safe.
pub const LIMIT: u32 = 3;
/// A launch that lasts this long counts as a good one.
pub const SURVIVED_SECS: u64 = 30;

pub struct LaunchGuard {
    file: PathBuf,
}

impl LaunchGuard {
    pub fn in_capsule(capsule: &Path) -> Self {
        Self {
            file: capsule.join("reports").join("launches"),
        }
    }

    /// Count this launch. Returns how many launches before it ended early,
    /// in a row; at [`LIMIT`] or more the window should start safe.
    pub fn begin(&self) -> u32 {
        let before = std::fs::read_to_string(&self.file)
            .ok()
            .and_then(|t| t.trim().parse::<u32>().ok())
            .unwrap_or(0);
        if let Some(dir) = self.file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&self.file, (before + 1).to_string());
        before
    }

    /// This launch got far enough: the count starts again.
    pub fn survived(&self) {
        let _ = std::fs::write(&self.file, "0");
    }
}

/// Whether launches that ended early this often mean a safe start.
pub fn safe_start(ended_early: u32) -> bool {
    ended_early >= LIMIT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_early_ends_in_a_row_start_safe_and_a_good_launch_resets() {
        let capsule = tempfile::tempdir().unwrap();
        let guard = LaunchGuard::in_capsule(capsule.path());
        let counts: Vec<u32> = (0..4).map(|_| guard.begin()).collect();
        assert_eq!(counts, [0, 1, 2, 3]);
        assert!(!safe_start(counts[2]));
        assert!(safe_start(counts[3]));
        guard.survived();
        assert_eq!(guard.begin(), 0);
    }
}
