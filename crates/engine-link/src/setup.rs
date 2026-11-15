//! The Capsule on first launch: create it if it is missing, and remember
//! when the first-run flow has finished.
//!
//! Called once before the link starts, like the login agent's sync, so
//! nothing here runs on the interface's foreground thread. The marker is
//! written from a background task.

use std::io;
use std::path::{Path, PathBuf};

/// Present once the first-run flow has finished.
pub const DONE_MARKER: &str = "settings/setup-done";

/// What the window found at launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapsuleState {
    /// The Capsule directory did not exist and was created now.
    pub created: bool,
    /// The first-run flow finished on an earlier launch.
    pub set_up: bool,
}

/// The Capsule directory and its first-run marker.
#[derive(Debug, Clone)]
pub struct Capsule {
    root: PathBuf,
}

impl Capsule {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create the Capsule directory if it is missing (user-only on Unix),
    /// and say whether first run has finished.
    pub fn prepare(&self) -> io::Result<CapsuleState> {
        let created = !self.root.exists();
        if created {
            create_private_dir(&self.root)?;
        }
        Ok(CapsuleState {
            created,
            set_up: self.root.join(DONE_MARKER).exists(),
        })
    }

    /// Remember that first run has finished.
    pub fn mark_set_up(&self) -> io::Result<()> {
        let marker = self.root.join(DONE_MARKER);
        if let Some(dir) = marker.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(marker, b"")
    }
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt as _;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
}

#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_capsule_is_created_and_first_run_is_remembered() {
        let home = tempfile::tempdir().unwrap();
        let capsule = Capsule::new(home.path().join("Isonomic"));

        let first = capsule.prepare().unwrap();
        assert_eq!(
            first,
            CapsuleState {
                created: true,
                set_up: false
            }
        );
        assert!(capsule.root().is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(capsule.root())
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700);
        }

        let again = capsule.prepare().unwrap();
        assert_eq!(
            again,
            CapsuleState {
                created: false,
                set_up: false
            }
        );

        capsule.mark_set_up().unwrap();
        let after = capsule.prepare().unwrap();
        assert_eq!(
            after,
            CapsuleState {
                created: false,
                set_up: true
            }
        );
    }
}
