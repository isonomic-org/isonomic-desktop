//! The acceptance check a person runs from the app
//! (`packaging/macos/acceptance-check.sh`), against a stand-in window: a
//! scratch Capsule in the temporary folder, keys in memory, the relaunch
//! given the first run's state, and nothing left behind, pass or fail.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{selfcheck, util};

/// A stand-in for the window: checks what the script gives it, and answers
/// as the window does.
const WINDOW: &str = r#"#!/bin/sh
echo "$ISONOMIC_CAPSULE_DIR" >> "$SEEN"
mkdir -p "$ISONOMIC_CAPSULE_DIR/logs"
grep -q '"scratch": true' "$ISONOMIC_CAPSULE_DIR/capsule.json" || exit 3
[ "$ISONOMIC_KEY_STORE" = memory ] || exit 4
echo "keys kept in memory: scratch Capsule" >> "$ISONOMIC_CAPSULE_DIR/logs/engine.log"
case "$ISONOMIC_SELF_CHECK" in
link)
    [ "$1" = "isonomic://chat/home" ] || exit 5
    [ "$ISONOMIC_SELF_CHECK_EXPECT" = "id=did:iso:x chats=1 chat=s1 turns=2" ] || exit 6
    echo "[self-check] 0.5s link: PASS" >&2 ;;
*)
    [ "$ISONOMIC_SELF_CHECK_DOWNLOAD" = 1 ] || exit 7
    echo "[self-check] 0.1s prompt: $ISONOMIC_SELF_CHECK" >&2
    echo "[self-check] 1.0s state id=did:iso:x chats=1 chat=s1 turns=2" >&2
    if [ -n "${BREAK:-}" ]; then
        echo "the engine broke" >> "$ISONOMIC_CAPSULE_DIR/logs/engine.log"
        exit 1
    fi ;;
esac
"#;

/// Run the script on a stand-in window; its output, whether it passed, and
/// the Capsules the window was given.
fn run(dir: &Path, broken: bool) -> (String, bool, Vec<PathBuf>) {
    use std::os::unix::fs::PermissionsExt as _;
    let window = dir.join("window");
    std::fs::write(&window, WINDOW).unwrap();
    std::fs::set_permissions(&window, std::fs::Permissions::from_mode(0o755)).unwrap();
    let seen = dir.join("seen");
    let tmp = dir.join("tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let mut command = Command::new("sh");
    command
        .arg(util::desktop_root().join(selfcheck::SCRIPT))
        .env("ISONOMIC_CHECK_WINDOW", &window)
        .env("TMPDIR", &tmp)
        .env("HOME", dir)
        .env("SEEN", &seen)
        .env_remove("ISONOMIC_CHECK_MODELS");
    if broken {
        command.env("BREAK", "1");
    }
    let out = command.output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let capsules = std::fs::read_to_string(&seen)
        .unwrap_or_default()
        .lines()
        .map(PathBuf::from)
        .collect();
    (text, out.status.success(), capsules)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("check-script-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_check_runs_twice_on_one_scratch_capsule_and_removes_it() {
    let dir = scratch("pass");
    let (out, passed, capsules) = run(&dir, false);
    assert!(passed, "{out}");
    assert!(
        out.contains("[self-check] 0.1s prompt: Plan one day in Lisbon"),
        "{out}"
    );
    assert!(out.contains("[self-check] 0.5s link: PASS"), "{out}");
    assert!(out.contains("keys: memory (scratch Capsule)"), "{out}");
    assert!(
        out.contains("acceptance check: PASS (prompt and link)"),
        "{out}"
    );
    assert_eq!(capsules.len(), 2, "{out}");
    assert_eq!(
        capsules[0], capsules[1],
        "the relaunch is on the same Capsule"
    );
    assert!(capsules[0].starts_with(dir.join("tmp")));
    assert!(!capsules[0].exists(), "the scratch Capsule is removed");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_run_explains_itself_and_still_leaves_nothing() {
    let dir = scratch("fail");
    let (out, passed, capsules) = run(&dir, true);
    assert!(!passed, "{out}");
    assert!(out.contains("--- last lines of logs/engine.log"), "{out}");
    assert!(out.contains("the engine broke"), "{out}");
    assert!(
        out.contains("acceptance check: FAIL (prompt failed, link failed)"),
        "{out}"
    );
    assert_eq!(capsules.len(), 1, "no relaunch after a failed first run");
    assert!(!capsules[0].exists(), "the scratch Capsule is removed");
    let _ = std::fs::remove_dir_all(&dir);
}
