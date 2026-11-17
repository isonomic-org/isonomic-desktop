// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hangs: the window's foreground thread beats every second; a thread here
//! writes a `hang` report when no beat has come for [`LIMIT_SECS`], and adds
//! how long it was stuck once beats resume. Rust cannot read another
//! thread's stack, so a hang report has no backtrace: it has the tab on
//! screen, the log's last lines and the time stuck.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::panic::Context;
use crate::{Component, Kind, Report};

/// No beat for this long is a hang.
pub const LIMIT_SECS: u64 = 5;
/// How often the watchdog looks.
pub const LOOK_EVERY: Duration = Duration::from_millis(500);

/// The foreground thread's side: call [`Beat::beat`] every second.
#[derive(Clone)]
pub struct Beat {
    started: Instant,
    last: Arc<AtomicU64>,
}

impl Beat {
    pub fn beat(&self) {
        self.last
            .store(self.started.elapsed().as_millis() as u64, Ordering::SeqCst);
    }

    fn since_last(&self) -> Duration {
        let now = self.started.elapsed().as_millis() as u64;
        Duration::from_millis(now.saturating_sub(self.last.load(Ordering::SeqCst)))
    }
}

/// What the watchdog does after each look. Pure, so tests drive it.
#[derive(Debug, Default)]
pub struct Watch {
    open: Option<(std::path::PathBuf, Duration)>,
}

impl Watch {
    /// Look once, with `quiet` since the last beat.
    pub fn look(&mut self, quiet: Duration, limit: Duration, context: &Context) {
        match (&self.open, quiet >= limit) {
            (None, true) => {
                let report = Report::new(
                    Kind::Hang,
                    Component::Window,
                    format!(
                        "the window stopped responding for at least {} s",
                        limit.as_secs()
                    ),
                );
                if let Ok(path) = context.save(report) {
                    self.open = Some((path, quiet));
                }
            }
            (Some((path, longest)), true) => self.open = Some((path.clone(), quiet.max(*longest))),
            (Some((path, longest)), false) => {
                // It recovered: say for how long it was stuck.
                if let Some(mut doc) = std::fs::read_to_string(path)
                    .ok()
                    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                {
                    doc["stuck_secs"] = serde_json::json!(longest.as_secs());
                    if let Ok(text) = serde_json::to_string_pretty(&doc) {
                        let _ = std::fs::write(path, text);
                    }
                }
                self.open = None;
            }
            (None, false) => {}
        }
    }
}

/// Start the watchdog thread; beat the returned handle every second.
pub fn start(context: Arc<Context>) -> Beat {
    let beat = Beat {
        started: Instant::now(),
        last: Arc::new(AtomicU64::new(0)),
    };
    let watched = beat.clone();
    let limit = Duration::from_secs(LIMIT_SECS);
    let _ = std::thread::Builder::new()
        .name("watchdog".into())
        .spawn(move || {
            let mut watch = Watch::default();
            loop {
                std::thread::sleep(LOOK_EVERY);
                watch.look(watched.since_last(), limit, &context);
            }
        });
    beat
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use crate::{Build, System};
    use std::sync::Mutex;

    #[test]
    fn a_hang_is_reported_once_and_its_length_added_when_it_ends() {
        let capsule = tempfile::tempdir().unwrap();
        let context = Context {
            store: Store::in_capsule(capsule.path()),
            build: Build::default(),
            system: Mutex::new(System::default()),
            home: None,
            apps: vec![],
            tab: Mutex::new(Some("com.isonomic.chat".into())),
            started: Instant::now(),
        };
        let limit = Duration::from_secs(5);
        let mut watch = Watch::default();
        watch.look(Duration::from_secs(1), limit, &context);
        assert!(context.store.pending().is_empty());
        watch.look(Duration::from_secs(5), limit, &context);
        watch.look(Duration::from_secs(9), limit, &context);
        assert_eq!(context.store.pending().len(), 1, "one report per hang");
        watch.look(Duration::from_millis(100), limit, &context);
        let doc = &context.store.pending()[0].document;
        assert_eq!(doc["kind"], "hang");
        assert_eq!(doc["stuck_secs"], 9);
        assert_eq!(doc["tab"], "com.isonomic.chat");
        assert_eq!(doc["backtrace"], serde_json::json!([]));
    }
}
