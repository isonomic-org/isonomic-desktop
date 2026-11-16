//! The window's own log: `logs/window.log` in the Capsule, also written to
//! standard error. It records state changes and errors, never what the
//! person types or what a model says. It rotates at [`ROTATE_AT`], keeping
//! one previous file, and a report carries its last lines.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// The log is rotated past this size.
pub const ROTATE_AT: u64 = 1 << 20;

pub struct WindowLog {
    path: PathBuf,
    lock: Mutex<()>,
}

static GLOBAL: OnceLock<WindowLog> = OnceLock::new();

impl WindowLog {
    pub fn in_capsule(capsule: &Path) -> Self {
        Self {
            path: capsule.join("logs").join("window.log"),
            lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Make this the log [`line`] writes to. The first call wins.
    pub fn install(self) -> &'static WindowLog {
        let _ = GLOBAL.set(self);
        GLOBAL.get().expect("set above")
    }

    pub fn write(&self, text: &str) {
        let _guard = self.lock.lock();
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if std::fs::metadata(&self.path).is_ok_and(|m| m.len() > ROTATE_AT) {
            let _ = std::fs::rename(&self.path, self.path.with_extension("log.1"));
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{text}");
        }
    }

    /// The last `n` lines, oldest first.
    pub fn tail(&self, n: usize) -> Vec<String> {
        let text = std::fs::read_to_string(&self.path).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        lines[lines.len().saturating_sub(n)..]
            .iter()
            .map(|l| l.to_string())
            .collect()
    }
}

/// Write one line to standard error and, once installed, the window log.
pub fn line(text: &str) {
    eprintln!("[isonomic-desktop] {text}");
    if let Some(log) = GLOBAL.get() {
        log.write(text);
    }
}

/// The installed log, for a report's tail.
pub fn installed() -> Option<&'static WindowLog> {
    GLOBAL.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_keeps_lines_rotates_and_gives_its_tail() {
        let capsule = tempfile::tempdir().unwrap();
        let log = WindowLog::in_capsule(capsule.path());
        for i in 0..60 {
            log.write(&format!("line {i}"));
        }
        let tail = log.tail(50);
        assert_eq!(tail.len(), 50);
        assert_eq!(tail.last().unwrap(), "line 59");
        std::fs::write(log.path(), vec![b'x'; (ROTATE_AT + 1) as usize]).unwrap();
        log.write("after");
        assert_eq!(log.tail(5), ["after"]);
        assert!(log.path().with_extension("log.1").is_file());
    }
}
