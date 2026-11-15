//! The login agent against a recording stand-in for `launchctl`.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use engine_link::agent::{in_app_bundle, AgentChange, Launchctl, LoginAgent, OFF_MARKER};

#[derive(Default)]
struct Recorded {
    calls: Mutex<Vec<String>>,
    loaded: AtomicBool,
}

impl Recorded {
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.calls.lock().unwrap())
    }
}

impl Launchctl for Recorded {
    fn run(&self, args: &[String]) -> io::Result<bool> {
        let verb = args[0].clone();
        self.calls.lock().unwrap().push(args.join(" "));
        Ok(match verb.as_str() {
            "print" => self.loaded.load(Ordering::SeqCst),
            "bootstrap" => {
                self.loaded.store(true, Ordering::SeqCst);
                true
            }
            "bootout" => {
                self.loaded.store(false, Ordering::SeqCst);
                true
            }
            _ => true,
        })
    }
}

fn engine_in(app: &Path) -> PathBuf {
    app.join("Isonomic.app/Contents/MacOS/isonomic-capsule")
}

fn agent(dir: &Path, engine: PathBuf, launchctl: &Arc<Recorded>) -> LoginAgent {
    LoginAgent::new(
        dir.join("LaunchAgents"),
        dir.join("capsule"),
        engine,
        501,
        Arc::clone(launchctl) as Arc<dyn Launchctl>,
    )
}

#[test]
fn the_first_launch_installs_and_loads_the_agent() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    let engine = engine_in(&dir.path().join("Applications"));
    let agent = agent(dir.path(), engine.clone(), &launchctl);

    assert_eq!(agent.sync().unwrap(), AgentChange::Installed);
    assert!(agent.enabled());
    let plist = std::fs::read_to_string(agent.plist_path()).unwrap();
    assert!(plist.contains("<string>com.isonomic.engine</string>"));
    assert!(plist.contains(&format!("<string>{}</string>", engine.display())));
    assert!(plist.contains("<key>ISONOMIC_CAPSULE_DIR</key>"));
    assert!(plist.contains("<key>SuccessfulExit</key>\n\t\t<false/>"));
    assert!(plist.contains("<key>RunAtLoad</key>\n\t<true/>"));
    assert!(
        dir.path().join("capsule/logs").is_dir(),
        "launchd opens the log"
    );
    assert_eq!(
        launchctl.take(),
        [
            "print gui/501/com.isonomic.engine".to_string(),
            format!("bootstrap gui/501 {}", agent.plist_path().display()),
        ]
    );

    // Every later launch: nothing to do.
    assert_eq!(agent.sync().unwrap(), AgentChange::Unchanged);
    assert_eq!(launchctl.take(), ["print gui/501/com.isonomic.engine"]);
}

#[test]
fn a_moved_app_reloads_the_agent_with_the_new_engine() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    agent(
        dir.path(),
        engine_in(&dir.path().join("Downloads")),
        &launchctl,
    )
    .sync()
    .unwrap();
    launchctl.take();

    let moved = agent(
        dir.path(),
        engine_in(&dir.path().join("Applications")),
        &launchctl,
    );
    assert_eq!(moved.sync().unwrap(), AgentChange::Installed);
    let calls = launchctl.take();
    assert_eq!(calls[1], "bootout gui/501/com.isonomic.engine");
    assert!(calls[2].starts_with("bootstrap gui/501 "));
    let plist = std::fs::read_to_string(moved.plist_path()).unwrap();
    assert!(plist.contains("Applications/Isonomic.app"));
}

#[test]
fn turning_it_off_is_remembered_and_leaves_the_running_engine_alone() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    let agent = agent(dir.path(), engine_in(dir.path()), &launchctl);
    agent.sync().unwrap();
    launchctl.take();

    assert_eq!(agent.set_enabled(false).unwrap(), AgentChange::Removed);
    assert!(!agent.enabled());
    assert!(dir.path().join("capsule").join(OFF_MARKER).exists());
    assert!(
        launchctl.take().is_empty(),
        "no bootout under the open window"
    );

    // The next launch keeps it off, and the window spawns the engine itself.
    assert_eq!(agent.sync().unwrap(), AgentChange::Unchanged);
    assert!(!agent.enabled());
    assert!(!agent.kickstart().unwrap());

    assert_eq!(agent.set_enabled(true).unwrap(), AgentChange::Installed);
    assert!(agent.enabled());
    assert!(!dir.path().join("capsule").join(OFF_MARKER).exists());
}

#[test]
fn the_window_starts_a_missing_engine_through_the_agent() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    let agent = agent(dir.path(), engine_in(dir.path()), &launchctl);
    agent.sync().unwrap();
    launchctl.take();

    assert!(agent.kickstart().unwrap());
    assert_eq!(
        launchctl.take(),
        [
            "print gui/501/com.isonomic.engine",
            "kickstart gui/501/com.isonomic.engine"
        ]
    );
}

#[test]
fn only_an_engine_inside_an_app_gets_an_agent() {
    assert!(in_app_bundle(Path::new(
        "/Applications/Isonomic.app/Contents/MacOS/isonomic-capsule"
    )));
    assert!(!in_app_bundle(Path::new(
        "/src/target/debug/isonomic-capsule"
    )));
}

#[test]
fn paths_are_escaped_in_the_plist() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    let odd = dir.path().join("R&D <apps>");
    let plist = agent(dir.path(), engine_in(&odd), &launchctl).render();
    assert!(plist.contains("R&amp;D &lt;apps&gt;"));
    assert!(!plist.contains("R&D"));
}

#[cfg(target_os = "macos")]
#[test]
fn the_plist_is_one_the_system_accepts() {
    let dir = tempfile::tempdir().unwrap();
    let launchctl = Arc::new(Recorded::default());
    let agent = agent(dir.path(), engine_in(&dir.path().join("R&D")), &launchctl);
    agent.sync().unwrap();
    let lint = std::process::Command::new("/usr/bin/plutil")
        .arg("-lint")
        .arg(agent.plist_path())
        .output()
        .unwrap();
    assert!(lint.status.success(), "{lint:?}");
}
