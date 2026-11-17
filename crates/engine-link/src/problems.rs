// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why the window can't reach the engine, in plain words.
//!
//! The link publishes the latest problem on a watch: the engine binary
//! could not be started, or it exited before it answered. A successful
//! connection clears it. Views subscribe; nothing polls.

use std::path::Path;
use std::sync::Arc;

use tokio::sync::watch;

/// The link's side of the problem watch. Cheap to clone.
#[derive(Clone)]
pub struct Problems {
    sender: Arc<watch::Sender<Option<String>>>,
}

impl Default for Problems {
    fn default() -> Self {
        let (sender, _) = watch::channel(None);
        Self {
            sender: Arc::new(sender),
        }
    }
}

impl Problems {
    /// The window's side: the latest problem, `None` once connected.
    pub fn watch(&self) -> watch::Receiver<Option<String>> {
        self.sender.subscribe()
    }

    pub fn report(&self, problem: String) {
        self.sender.send_replace(Some(problem));
    }

    pub fn clear(&self) {
        self.sender
            .send_if_modified(|current| current.take().is_some());
    }

    pub fn current(&self) -> Option<String> {
        self.sender.borrow().clone()
    }
}

/// The problem for an engine that could not be started at all.
pub fn not_started(binary: &Path, error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => format!(
            "The engine isn't installed where the window expects it ({}). Reinstall Isonomic.",
            binary.display()
        ),
        std::io::ErrorKind::PermissionDenied => format!(
            "The engine at {} isn't allowed to run. Reinstall Isonomic.",
            binary.display()
        ),
        _ => format!("The engine couldn't be started: {error}."),
    }
}

/// Of the engine's standard error at `stderr` and its own `engine.log`
/// beside it, the one written last: where its last words are. Before it
/// accepts the Capsule it has only standard error.
fn latest_log(stderr: &Path) -> std::path::PathBuf {
    let own = stderr.with_file_name("engine.log");
    let written = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    match (written(stderr), written(&own)) {
        (Some(err), Some(log)) if log > err => own,
        (None, Some(_)) => own,
        _ => stderr.to_path_buf(),
    }
}

/// The problem for an engine that exited before it answered, with the last
/// line it wrote, when there is one.
pub fn exited(status: &std::process::ExitStatus, log: Option<&Path>) -> String {
    let code = status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "stopped by a signal".into());
    let log = log.map(latest_log);
    let log = log.as_deref();
    let last = log
        .and_then(|log| std::fs::read_to_string(log).ok())
        .and_then(|text| {
            text.lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .map(str::to_string)
        });
    match (last, log) {
        (Some(line), Some(log)) => format!(
            "The engine stopped as it started ({code}). Its last words: \"{line}\". Full log: {}.",
            log.display()
        ),
        _ => format!("The engine stopped as it started ({code})."),
    }
}
