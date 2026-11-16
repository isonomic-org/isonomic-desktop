//! The window's side of problem reports at startup: its log, the panic
//! hook, the hang watchdog and the crash-loop guard. Reports are kept in
//! the Capsule; nothing is sent from this build.

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use reports::guard::{safe_start, LaunchGuard};
use reports::log::WindowLog;
use reports::panic::{AppCrate, Context};
use reports::store::Store;
use reports::watchdog::{self, Beat};
use reports::{Build, System};

/// What Home says after a safe start.
pub const SAFE_NOTICE: &str = "Isonomic stopped as it started three times in a row, so it opened \
without its apps this time. Quit and open it again to bring them back.";

pub struct Problems {
    pub context: Arc<Context>,
    pub guard: LaunchGuard,
    /// Start with no app mounted.
    pub safe: bool,
    pub beat: Beat,
}

fn build() -> Build {
    Build {
        version: env!("CARGO_PKG_VERSION").into(),
        build: env!("ISONOMIC_REPORT_BUILD").into(),
        arch: std::env::consts::ARCH.into(),
        sdk: env!("ISONOMIC_REPORT_SDK").into(),
        chat: env!("ISONOMIC_REPORT_CHAT").into(),
        engine: env!("ISONOMIC_REPORT_ENGINE").into(),
    }
}

fn macos_version() -> String {
    Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

/// The Capsule's format number, from its `capsule.json`.
fn capsule_format(root: &Path) -> Option<u32> {
    let text = std::fs::read_to_string(root.join("capsule.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value["format"].as_u64().map(|n| n as u32)
}

/// Before the window opens: count this launch, open the log, install the
/// hook and start the watchdog. Blocks briefly on the disk; call it before
/// the event loop. A self-check never starts safe.
pub fn start(root: &Path, apps: Vec<AppCrate>, check: bool) -> Problems {
    WindowLog::in_capsule(root).install();
    let guard = LaunchGuard::in_capsule(root);
    let ended_early = guard.begin();
    let safe = !check && safe_start(ended_early);
    let store = Store::in_capsule(root);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    store.prune(now.as_millis());
    let context = Arc::new(Context {
        store,
        build: build(),
        system: Mutex::new(System {
            os_version: macos_version(),
            capsule_format: capsule_format(root),
            ..System::default()
        }),
        home: std::env::var("HOME").ok(),
        apps,
        tab: Mutex::new(Some("home".into())),
        started: Instant::now(),
    });
    reports::panic::install(Arc::clone(&context));
    let beat = watchdog::start(Arc::clone(&context));
    if safe {
        reports::log::line(&format!(
            "safe start after {ended_early} early ends in a row"
        ));
    }
    Problems {
        context,
        guard,
        safe,
        beat,
    }
}

/// Record the machine's profile once the engine has said it.
pub fn profile_known(context: &Context, profile: &str, total_memory_bytes: u64) {
    if let Ok(mut system) = context.system.lock() {
        system.profile = Some(profile.to_string());
        system.memory_gb = Some((total_memory_bytes + (1 << 29)) >> 30);
    }
}
