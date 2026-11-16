//! What the engine says once at start that the person should see: today,
//! that a damaged list of apps and chats was set aside and what could
//! still be read was recovered (isonomic-capsule `registry::check`, with
//! its counts of chats and approvals). The engine says it only in its own log, so the
//! window reads the latest run's lines there and shows the message on
//! Home as a plain notice, once per incident.

use std::path::Path;

/// Home's title for it. The engine's message, as it stands, says what came
/// back and whether apps must be approved again.
pub const SET_ASIDE_TITLE: &str = "Isonomic recovered what it could";
/// How the engine's message starts (`SetAside::message`); the second is an
/// engine from before it recovered anything.
const SET_ASIDE: [&str; 2] = [
    "The list of apps and chats could not be read",
    "The list of approved apps could not be read",
];
/// The engine's first line in every run (`capsule directory: …`).
const RUN_START: &str = "capsule directory:";
/// Incidents already shown, one message per line, in the Capsule.
const SEEN: &str = "settings/seen-engine-notices";

/// The set-aside message from the engine's latest run in `log`, if any.
pub fn set_aside(log: &str) -> Option<String> {
    let lines: Vec<&str> = log.lines().collect();
    let run = lines
        .iter()
        .rposition(|l| l.contains(RUN_START))
        .unwrap_or(0);
    lines[run..].iter().find_map(|l| {
        SET_ASIDE
            .iter()
            .find_map(|start| l.find(start))
            .map(|at| l[at..].trim().to_string())
    })
}

/// The message to show for the Capsule at `root`, once: `None` when there
/// is none, or it has been shown before. Touches the disk; call it off the
/// foreground thread.
pub fn unseen(root: &Path) -> Option<String> {
    let logs = root.join("logs");
    // A rotated log keeps the start of a long run in `engine.log.1`.
    let log = ["engine.log.1", "engine.log"]
        .iter()
        .filter_map(|name| std::fs::read_to_string(logs.join(name)).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let message = set_aside(&log)?;
    let seen = root.join(SEEN);
    let before = std::fs::read_to_string(&seen).unwrap_or_default();
    if before.lines().any(|l| l == message) {
        return None;
    }
    if let Some(dir) = seen.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&seen, format!("{before}{message}\n"));
    Some(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MESSAGE: &str = "The list of approved apps could not be read (file is not a database), \
so it was set aside in /c/unreadable/registry-20261116T080000.db and a new one was started. \
Apps must be approved again. Their data and files are kept; chats from before are in the \
set-aside copy and are not shown.";

    /// The engine's message since it recovers what it can (isonomic-capsule #73).
    const RECOVERED: &str = "The list of apps and chats could not be read (database disk image is \
malformed). It was set aside in /c/unreadable/registry-20261116T100000.db, and what could still \
be read was copied into a new one: 297 chats recovered, 3 could not be read. Approved apps keep \
their approvals. Their data and files are kept, and what was not recovered stays in the \
set-aside copy.";

    #[test]
    fn the_recovering_engine_s_message_is_shown_as_it_stands() {
        let log = format!("capsule directory: /c\n[isonomic-capsule] {RECOVERED}\nready\n");
        assert_eq!(set_aside(&log).as_deref(), Some(RECOVERED));
    }

    #[test]
    fn only_the_latest_run_s_message_counts() {
        let log = format!("capsule directory: /c\n{MESSAGE}\ncapsule directory: /c\nready\n");
        assert_eq!(set_aside(&log), None, "an earlier run's");
        let log = format!("capsule directory: /c\nready\ncapsule directory: /c\n{MESSAGE}\n");
        assert_eq!(set_aside(&log).as_deref(), Some(MESSAGE));
        assert_eq!(set_aside(""), None);
    }

    #[test]
    fn each_incident_is_shown_once() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("logs")).unwrap();
        assert_eq!(unseen(root.path()), None);
        std::fs::write(
            root.path().join("logs/engine.log"),
            format!("capsule directory: /c\n{MESSAGE}\n"),
        )
        .unwrap();
        assert_eq!(unseen(root.path()).as_deref(), Some(MESSAGE));
        assert_eq!(unseen(root.path()), None, "shown once");
        let next = MESSAGE.replace("080000", "090000");
        std::fs::write(
            root.path().join("logs/engine.log"),
            format!("capsule directory: /c\n{next}\n"),
        )
        .unwrap();
        assert_eq!(unseen(root.path()), Some(next), "a new incident");
    }
}
