// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The panic hook: before the process ends, it writes a report.
//!
//! A panic inside a compiled-in app's own drawing or events unwinds
//! through the interface framework and ends the process; the hook blames
//! the app whose crate comes first in the backtrace, so the next launch can
//! say "Chat caused Isonomic to quit".

use std::panic::PanicHookInfo;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::backtrace::{frames, short_path};
use crate::store::Store;
use crate::{log, Build, Component, Kind, Report, System, LOG_LINES};

/// An app crate compiled into the window, to blame a panic on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppCrate {
    /// The crate's name in paths, such as `isonomic_chat`.
    pub prefix: String,
    pub id: String,
    pub version: String,
}

/// The crate a type lives in, for [`AppCrate::prefix`].
pub fn crate_of<T: ?Sized>() -> String {
    let name = std::any::type_name::<T>();
    name.split("::").next().unwrap_or(name).to_string()
}

/// What every report from this process shares.
pub struct Context {
    pub store: Store,
    pub build: Build,
    pub system: Mutex<System>,
    pub home: Option<String>,
    pub apps: Vec<AppCrate>,
    /// The tab on screen, kept up to date by the shell.
    pub tab: Mutex<Option<String>>,
    pub started: Instant,
}

impl Context {
    /// Write `report` as a document, with this process's build and system.
    pub fn save(&self, mut report: Report) -> std::io::Result<std::path::PathBuf> {
        if report.log_tail.is_empty() {
            report.log_tail = log::installed()
                .map(|l| l.tail(LOG_LINES))
                .unwrap_or_default();
        }
        if report.tab.is_none() {
            report.tab = self.tab.lock().ok().and_then(|t| t.clone());
        }
        report.uptime_secs = self.started.elapsed().as_secs();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let system = self.system.lock().map(|s| s.clone()).unwrap_or_default();
        let doc = report.to_json(&self.build, &system, self.home.as_deref(), now.as_secs());
        self.store.save(&doc, now.as_millis())
    }
}

thread_local! {
    static CONTAINED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run a call into an app, catching its panic: the window survives, and the
/// hook writes the report as `app_failed` rather than a crash. `Err`
/// carries the panic's message (the report itself is scrubbed).
pub fn contain<R>(call: impl FnOnce() -> R) -> Result<R, String> {
    let before = CONTAINED.with(|c| c.replace(true));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(call));
    CONTAINED.with(|c| c.set(before));
    outcome.map_err(|payload| {
        payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a panic with no message".into())
    })
}

/// Install the hook. The default hook still runs after it.
pub fn install(context: Arc<Context>) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = context.save(from_panic(info, &context.apps));
        previous(info);
    }));
}

fn from_panic(info: &PanicHookInfo<'_>, apps: &[AppCrate]) -> Report {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic with no message".into());
    let at = info
        .location()
        .map(|l| format!(" at {}", short_path(&format!("{}:{}", l.file(), l.line()))))
        .unwrap_or_default();
    let backtrace = frames(&std::backtrace::Backtrace::force_capture().to_string());
    let kind = if CONTAINED.with(|c| c.get()) {
        Kind::AppFailed
    } else {
        Kind::Panic
    };
    let mut report = Report::new(kind, blame(&backtrace, apps), format!("{message}{at}"));
    report.backtrace = backtrace;
    report
}

/// The app whose crate shows first in the frames, else the window.
pub fn blame(frames: &[String], apps: &[AppCrate]) -> Component {
    for frame in frames {
        let function = frame.trim_start_matches('<');
        if let Some(app) = apps
            .iter()
            .find(|a| function.starts_with(&format!("{}::", a.prefix)))
        {
            return Component::App {
                id: app.id.clone(),
                version: app.version.clone(),
            };
        }
    }
    Component::Window
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACE: &str = "   0: std::backtrace::Backtrace::force_capture
             at /rustc/abc/library/std/src/backtrace.rs:312:13
   1: reports::panic::from_panic
             at /Users/omar/Code/isonomic-desktop/crates/reports/src/panic.rs:70:21
   2: <isonomic_chat::shell::ChatShell as gpui::Render>::render
             at /Users/omar/Code/isonomic-chat/crates/chat/src/shell/mod.rs:212:9
   3: shell::Shell::render
             at /Users/omar/Code/isonomic-desktop/crates/shell/src/lib.rs:400:5";

    /// The panic hook is one per process: tests that set it take turns.
    static HOOK: Mutex<()> = Mutex::new(());

    fn chat() -> Vec<AppCrate> {
        vec![AppCrate {
            prefix: "isonomic_chat".into(),
            id: "com.isonomic.chat".into(),
            version: "0.1.0".into(),
        }]
    }

    #[test]
    fn frames_keep_functions_and_crate_relative_places_only() {
        let frames = frames(TRACE);
        assert_eq!(frames.len(), 4);
        assert_eq!(
            frames[0],
            "std::backtrace::Backtrace::force_capture (std/src/backtrace.rs:312)"
        );
        assert!(
            frames[2].ends_with("(chat/src/shell/mod.rs:212)"),
            "{}",
            frames[2]
        );
        assert!(frames.iter().all(|f| !f.contains("/Users/")));
    }

    #[test]
    fn a_panic_in_an_apps_crate_is_blamed_on_that_app() {
        let frames = frames(TRACE);
        assert_eq!(
            blame(&frames, &chat()),
            Component::App {
                id: "com.isonomic.chat".into(),
                version: "0.1.0".into()
            }
        );
        assert_eq!(blame(&frames[3..], &chat()), Component::Window);
    }

    #[test]
    fn the_hook_writes_a_report_before_the_process_ends() {
        let _turn = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        let capsule = tempfile::tempdir().unwrap();
        let context = Arc::new(Context {
            store: Store::in_capsule(capsule.path()),
            build: Build::default(),
            system: Mutex::new(System::default()),
            home: None,
            apps: chat(),
            tab: Mutex::new(Some("home".into())),
            started: Instant::now(),
        });
        install(Arc::clone(&context));
        let caught = std::panic::catch_unwind(|| panic!("the window fell over"));
        let _ = std::panic::take_hook();
        assert!(caught.is_err());
        let pending = context.store.pending();
        assert_eq!(pending.len(), 1);
        let doc = &pending[0].document;
        assert_eq!(doc["kind"], "panic");
        assert!(doc["message"]
            .as_str()
            .unwrap()
            .starts_with("the window fell over at reports/src/panic.rs:"));
        assert_eq!(doc["tab"], "home");
        assert!(doc["backtrace"].as_array().is_some_and(|b| !b.is_empty()));
    }

    #[test]
    fn a_contained_panic_is_an_app_failure_and_the_caller_goes_on() {
        let _turn = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        let capsule = tempfile::tempdir().unwrap();
        let context = Arc::new(Context {
            store: Store::in_capsule(capsule.path()),
            build: Build::default(),
            system: Mutex::new(System::default()),
            home: None,
            apps: vec![],
            tab: Mutex::new(None),
            started: Instant::now(),
        });
        install(Arc::clone(&context));
        let failed = contain(|| -> u32 { panic!("the app broke") });
        let fine = contain(|| 7);
        let _ = std::panic::take_hook();
        assert_eq!(failed, Err("the app broke".to_string()));
        assert_eq!(fine, Ok(7));
        let pending = context.store.pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].document["kind"], "app_failed");
    }

    #[test]
    fn a_type_names_its_crate() {
        assert_eq!(crate_of::<Context>(), "reports");
    }
}
