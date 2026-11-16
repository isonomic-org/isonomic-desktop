//! Problem and crash reports (design agreed in the Worker reports, 02:56 UTC).
//!
//! A report is one small JSON document, format 1, written to the Capsule's
//! `reports/pending/`. This crate captures and keeps reports; it sends
//! nothing. Where reports go is not decided, so nothing leaves the Mac.
//!
//! - [`scrub`]: what never goes in a report (home paths, long quoted text).
//! - [`Report`]: the document, with the build and system it came from.
//! - [`store::Store`]: pending reports and their retention.
//! - [`guard::LaunchGuard`]: the crash-loop guard.
//! - [`log::WindowLog`]: the window's own log, whose tail a report carries.
//! - [`panic`]: the panic hook; [`watchdog`]: hangs.

pub mod guard;
pub mod log;
pub mod panic;
pub mod scrub;
pub mod store;
pub mod watchdog;

use serde_json::{json, Map, Value};

/// The report format this build writes.
pub const FORMAT: u32 = 1;
/// How many lines of the window log a report carries.
pub const LOG_LINES: usize = 50;

/// What a report is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Panic,
    Hang,
    /// An app failed at a call the window made, and was contained.
    AppFailed,
    /// An app reported a handled problem.
    AppError,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Panic => "panic",
            Kind::Hang => "hang",
            Kind::AppFailed => "app_failed",
            Kind::AppError => "app_error",
        }
    }
}

/// The build a report comes from. Filled in at compile time by the window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Build {
    pub version: String,
    pub build: String,
    pub arch: String,
    /// The SDK, Chat and engine commits the build carries.
    pub sdk: String,
    pub chat: String,
    pub engine: String,
}

/// The machine a report comes from, as far as the window knows it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct System {
    pub macos: String,
    /// `low`, `standard` or `high`, once the engine has said.
    pub profile: Option<String>,
    pub memory_gb: Option<u64>,
    /// The Capsule's format number, from its `capsule.json`.
    pub capsule_format: Option<u32>,
}

/// Which part failed: the window, or an app with its version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    Window,
    App { id: String, version: String },
}

/// One report, before it is written.
#[derive(Debug, Clone)]
pub struct Report {
    pub kind: Kind,
    pub component: Component,
    pub message: String,
    pub backtrace: Vec<String>,
    pub log_tail: Vec<String>,
    /// The tab on screen when it happened.
    pub tab: Option<String>,
    pub uptime_secs: u64,
    pub extra: Map<String, Value>,
}

impl Report {
    pub fn new(kind: Kind, component: Component, message: impl Into<String>) -> Self {
        Self {
            kind,
            component,
            message: message.into(),
            backtrace: Vec::new(),
            log_tail: Vec::new(),
            tab: None,
            uptime_secs: 0,
            extra: Map::new(),
        }
    }

    /// The document as written: every text field scrubbed against `home`.
    pub fn to_json(
        &self,
        build: &Build,
        system: &System,
        home: Option<&str>,
        unix_secs: u64,
    ) -> Value {
        let component = match &self.component {
            Component::Window => json!({ "id": "window" }),
            Component::App { id, version } => json!({ "id": id, "version": version }),
        };
        let mut doc = json!({
            "format": FORMAT,
            "kind": self.kind.as_str(),
            "component": component,
            "product": {
                "version": build.version,
                "build": build.build,
                "arch": build.arch,
                "pins": { "sdk": build.sdk, "chat": build.chat, "engine": build.engine },
            },
            "system": {
                "macos": system.macos,
                "profile": system.profile,
                "memory_gb": system.memory_gb,
                "capsule_format": system.capsule_format,
            },
            "when": { "unix_secs": unix_secs, "uptime_secs": self.uptime_secs },
            "message": scrub::message(&self.message, home),
            "backtrace": self.backtrace.iter().map(|l| scrub::line(l, home)).collect::<Vec<_>>(),
            "log_tail": self.log_tail.iter().map(|l| scrub::line(l, home)).collect::<Vec<_>>(),
            "tab": self.tab,
        });
        if let Value::Object(map) = &mut doc {
            for (key, value) in &self.extra {
                map.insert(key.clone(), value.clone());
            }
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_names_its_build_pins_system_and_capsule_format() {
        let build = Build {
            version: "0.9.0".into(),
            build: "214".into(),
            arch: "x86_64".into(),
            sdk: "439a069".into(),
            chat: "9b58161".into(),
            engine: "506097d".into(),
        };
        let system = System {
            macos: "15.1".into(),
            profile: Some("standard".into()),
            memory_gb: Some(16),
            capsule_format: Some(1),
        };
        let mut report = Report::new(
            Kind::AppFailed,
            Component::App {
                id: "com.isonomic.chat".into(),
                version: "0.1.0".into(),
            },
            "build failed at /Users/omar/Library/x",
        );
        report.tab = Some("com.isonomic.chat".into());
        let doc = report.to_json(&build, &system, Some("/Users/omar"), 1_700_000_000);
        assert_eq!(doc["format"], FORMAT);
        assert_eq!(doc["kind"], "app_failed");
        assert_eq!(doc["component"]["id"], "com.isonomic.chat");
        assert_eq!(doc["product"]["pins"]["chat"], "9b58161");
        assert_eq!(doc["system"]["capsule_format"], 1);
        assert_eq!(doc["message"], "build failed at ~/Library/x");
    }
}
