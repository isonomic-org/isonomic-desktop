//! Start at login (product spec, lifecycle): a per-user launch agent that
//! starts the engine at login with no window, and again if it crashes.
//!
//! The agent restarts the engine only after an unsuccessful exit. Quit asks
//! the engine to stop, it exits 0, and the agent leaves it stopped; a second
//! engine started while one serves also exits 0 (engine #21), so nothing
//! loops.
//!
//! The plist names the engine inside the app and pins the Capsule directory,
//! because a launch agent's environment is not the user's shell. Turning the
//! agent off is remembered in the Capsule directory, so a later launch
//! doesn't turn it back on.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use isonomic_proto::locate;

/// The agent's label, and the plist's file name.
pub const LABEL: &str = "com.isonomic.engine";
/// Present when the person turned start at login off.
pub const OFF_MARKER: &str = "settings/start-at-login-off";

/// Runs `launchctl`. The real one runs the program; tests record calls.
pub trait Launchctl: Send + Sync + 'static {
    /// Run with these arguments; `Ok(true)` when it exits 0.
    fn run(&self, args: &[String]) -> io::Result<bool>;
}

/// `/bin/launchctl`.
pub struct SystemLaunchctl;

impl Launchctl for SystemLaunchctl {
    fn run(&self, args: &[String]) -> io::Result<bool> {
        let status = std::process::Command::new("/bin/launchctl")
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?;
        Ok(status.success())
    }
}

/// What `sync` did, for the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentChange {
    /// Already as it should be.
    Unchanged,
    /// Written for the first time, or rewritten because the app moved.
    Installed,
    /// Turned off: the plist is gone.
    Removed,
}

pub struct LoginAgent {
    plist: PathBuf,
    root: PathBuf,
    engine: PathBuf,
    user_id: u32,
    launchctl: Arc<dyn Launchctl>,
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Is this engine inside an app bundle? Builds run from a source tree don't
/// install an agent: it would point into a build directory.
pub fn in_app_bundle(engine: &Path) -> bool {
    engine
        .parent()
        .map(|dir| dir.ends_with("Contents/MacOS"))
        .unwrap_or(false)
}

impl LoginAgent {
    pub fn new(
        launch_agents: PathBuf,
        root: PathBuf,
        engine: PathBuf,
        user_id: u32,
        launchctl: Arc<dyn Launchctl>,
    ) -> Self {
        Self {
            plist: launch_agents.join(format!("{LABEL}.plist")),
            root,
            engine,
            user_id,
            launchctl,
        }
    }

    pub fn plist_path(&self) -> &Path {
        &self.plist
    }

    fn domain(&self) -> String {
        format!("gui/{}", self.user_id)
    }

    fn service(&self) -> String {
        format!("gui/{}/{LABEL}", self.user_id)
    }

    fn engine_log(&self) -> PathBuf {
        self.root.join("logs").join("engine.log")
    }

    /// The plist for this engine and Capsule.
    pub fn render(&self) -> String {
        let engine = xml_escape(&self.engine.to_string_lossy());
        let root = xml_escape(&self.root.to_string_lossy());
        let log = xml_escape(&self.engine_log().to_string_lossy());
        let env = locate::ENV_CAPSULE_DIR;
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{LABEL}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{engine}</string>
	</array>
	<key>EnvironmentVariables</key>
	<dict>
		<key>{env}</key>
		<string>{root}</string>
	</dict>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<dict>
		<key>SuccessfulExit</key>
		<false/>
	</dict>
	<key>LimitLoadToSessionType</key>
	<string>Aqua</string>
	<key>StandardErrorPath</key>
	<string>{log}</string>
</dict>
</plist>
"#
        )
    }

    /// Has the person turned start at login off?
    pub fn turned_off(&self) -> bool {
        self.root.join(OFF_MARKER).exists()
    }

    /// Is the plist in place? This is what the menu shows.
    pub fn enabled(&self) -> bool {
        self.plist.exists()
    }

    fn loaded(&self) -> bool {
        self.launchctl
            .run(&["print".into(), self.service()])
            .unwrap_or(false)
    }

    /// Bring the agent in line with the person's choice: on by default,
    /// off once they turned it off. Run at every launch, so a moved app
    /// gets its plist rewritten. Blocks on file and `launchctl` calls; keep
    /// it off the foreground thread.
    pub fn sync(&self) -> io::Result<AgentChange> {
        if self.turned_off() {
            return self.remove();
        }
        self.install()
    }

    /// Turn start at login on or off, and remember the choice.
    pub fn set_enabled(&self, on: bool) -> io::Result<AgentChange> {
        let marker = self.root.join(OFF_MARKER);
        if on {
            match std::fs::remove_file(&marker) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
                _ => {}
            }
            self.install()
        } else {
            if let Some(dir) = marker.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&marker, b"")?;
            self.remove()
        }
    }

    fn install(&self) -> io::Result<AgentChange> {
        let wanted = self.render();
        let current = std::fs::read_to_string(&self.plist).ok();
        let changed = current.as_deref() != Some(wanted.as_str());
        if changed {
            if let Some(dir) = self.plist.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::create_dir_all(self.root.join("logs"))?;
            std::fs::write(&self.plist, &wanted)?;
        }
        let loaded = self.loaded();
        if changed && loaded {
            // The app moved: the running job still names the old engine.
            self.launchctl.run(&["bootout".into(), self.service()])?;
        }
        if changed || !loaded {
            self.launchctl.run(&[
                "bootstrap".into(),
                self.domain(),
                self.plist.to_string_lossy().into_owned(),
            ])?;
        }
        Ok(if changed {
            AgentChange::Installed
        } else {
            AgentChange::Unchanged
        })
    }

    /// Delete the plist. The job loaded for this login session is left
    /// alone: unloading it would stop the engine under the open window. It
    /// is gone from the next login.
    fn remove(&self) -> io::Result<AgentChange> {
        match std::fs::remove_file(&self.plist) {
            Ok(()) => Ok(AgentChange::Removed),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(AgentChange::Unchanged),
            Err(error) => Err(error),
        }
    }

    /// Start the engine through the agent, when it's installed. The window
    /// does this instead of spawning the binary, so the engine it starts is
    /// one the agent restarts after a crash.
    pub fn kickstart(&self) -> io::Result<bool> {
        if !self.enabled() {
            return Ok(false);
        }
        if !self.loaded() {
            self.launchctl.run(&[
                "bootstrap".into(),
                self.domain(),
                self.plist.to_string_lossy().into_owned(),
            ])?;
        }
        self.launchctl.run(&["kickstart".into(), self.service()])
    }
}
