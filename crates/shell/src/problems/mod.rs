//! Problem reports, as the person sees them (problem-report design,
//! section 3): a notice for each kept report, the review of its exact
//! bytes, and the "Problem reports" setting on Home.
//!
//! This build has no sender and no destination, so Send is never offered:
//! the notice says the report is kept on this Mac and nothing leaves it.
//! Every disk call goes through [`Kept`] on the background executor.

mod notice;
mod setting;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use reports::kept::{self, Kept};
use reports::settings::{Mode, ReportSettings};
use reports::store::Pending;
use ui::{AppContext as _, Context, Entity, Window};

use crate::Shell;

pub use setting::row as setting_row;

pub struct ProblemReports {
    kept: Arc<Kept>,
    settings: ReportSettings,
    /// The setting has been read from the Capsule, or chosen here.
    loaded: bool,
    /// The report the notice is about.
    current: Option<Pending>,
    /// Reports the person kept this session: not asked about again.
    asked: HashSet<PathBuf>,
    /// The person's Isonomic ID, once the engine has given it.
    identity: Option<String>,
    /// The notice's checkbox, off for every new notice.
    include_id: bool,
    /// App names by id, for titles such as "Chat stopped working".
    names: HashMap<String, String>,
    /// Writes reports apps hand the window, with this process's build.
    writer: Option<Arc<reports::panic::Context>>,
}

impl ProblemReports {
    pub fn new(kept: Arc<Kept>, cx: &mut Context<Self>) -> Self {
        let loading = Arc::clone(&kept);
        cx.spawn(async move |this, cx| {
            let settings = cx.background_spawn(async move { loading.settings() }).await;
            let _ = this.update(cx, |this, cx| {
                if !this.loaded {
                    (this.settings, this.loaded) = (settings, true);
                    cx.notify();
                }
            });
        })
        .detach();
        Self {
            kept,
            settings: ReportSettings::default(),
            loaded: false,
            current: None,
            asked: HashSet::new(),
            identity: None,
            include_id: false,
            names: HashMap::new(),
            writer: None,
        }
    }

    /// Write apps' reported problems with `writer`: the same build, system
    /// and store the panic hook uses.
    pub fn write_with(&mut self, writer: Arc<reports::panic::Context>) {
        self.writer = Some(writer);
    }

    /// Titles name the app with `id` as `name`.
    pub fn name_app(&mut self, id: &str, name: &str) {
        self.names.insert(id.to_string(), name.to_string());
    }

    /// The person's Isonomic ID: the notice offers to include it.
    pub fn set_identity(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.identity = id;
        cx.notify();
    }

    pub fn mode(&self) -> Mode {
        self.settings.mode
    }

    /// The report the notice on screen is about.
    pub fn current(&self) -> Option<&Pending> {
        self.current.as_ref()
    }

    /// Send needs a destination and a sender; this build has neither.
    pub fn offers_send(&self) -> bool {
        false
    }

    /// The checkbox is there only once the ID is known.
    pub fn offers_id(&self) -> bool {
        self.identity.is_some()
    }

    pub fn includes_id(&self) -> bool {
        self.include_id
    }

    /// Exactly what would be sent for the report on screen.
    pub fn outgoing(&self) -> Option<String> {
        let id = self.identity.as_deref().filter(|_| self.include_id);
        self.current
            .as_ref()
            .map(|report| kept::outgoing(report, id))
    }

    /// Look at the kept reports and act on the setting: ask about the
    /// oldest not yet asked, keep them all (Send automatically, with
    /// nowhere to send), or delete them (Never send).
    pub fn review(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.current.is_some() {
            return; // a notice is already on screen; the next follows it
        }
        let kept = Arc::clone(&self.kept);
        // Until the setting has loaded, read it here: a review at launch
        // must not ask when the person chose Never send.
        let known = self.loaded.then_some(self.settings.mode);
        let asked = self.asked.clone();
        cx.spawn_in(window, async move |this, cx| {
            let next = cx
                .background_spawn(async move {
                    let mode = known.unwrap_or_else(|| kept.settings().mode);
                    let pending = kept.pending();
                    match mode {
                        Mode::Ask => pending.into_iter().find(|r| !asked.contains(&r.path)),
                        Mode::Always => None,
                        Mode::Never => {
                            for report in &pending {
                                let _ = kept.discard(report);
                            }
                            None
                        }
                    }
                })
                .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.current.is_some() || next.is_none() {
                    return;
                }
                this.current = next;
                this.include_id = false;
                notice::open(cx.entity(), window, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Keep the report on this Mac, then look at the next.
    pub fn keep(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(report) = self.current.take() else {
            return;
        };
        self.asked.insert(report.path);
        cx.notify();
        self.review(window, cx);
    }

    /// Don't send: the report is deleted, then the next is looked at.
    pub fn dont_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(report) = self.current.take() else {
            return;
        };
        cx.notify();
        let kept = Arc::clone(&self.kept);
        cx.spawn_in(window, async move |this, cx| {
            let _ = cx
                .background_spawn(async move { kept.discard(&report) })
                .await;
            let _ = this.update_in(cx, |this, window, cx| this.review(window, cx));
        })
        .detach();
    }

    pub fn toggle_id(&mut self, cx: &mut Context<Self>) {
        self.include_id = !self.include_id && self.identity.is_some();
        cx.notify();
    }

    /// Change the setting; it is kept in the Capsule.
    pub fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if self.settings.mode == mode {
            return;
        }
        (self.settings.mode, self.loaded) = (mode, true);
        cx.notify();
        let (kept, settings) = (Arc::clone(&self.kept), self.settings.clone());
        cx.background_spawn(async move {
            if let Err(error) = kept.save_settings(&settings) {
                reports::log::line(&format!("report setting not saved: {}", error.kind()));
            }
        })
        .detach();
    }

    /// The notice's title, for the report on screen.
    pub fn title(&self) -> String {
        let Some(report) = &self.current else {
            return String::new();
        };
        title_for(&report.document, &self.names)
    }
}

/// A notice's title for `document`, written by the window or the engine.
fn title_for(document: &serde_json::Value, names: &HashMap<String, String>) -> String {
    let id = document["component"]["id"].as_str().unwrap_or("window");
    match (document["kind"].as_str(), id) {
        (Some("service_failed"), _) => {
            let name = document["service"]["name"].as_str().unwrap_or("");
            format!("{} stopped working", service_name(name))
        }
        (Some("model_server_failed"), _) => "The local model stopped working".into(),
        (Some("hang"), "window") => "Isonomic stopped responding".into(),
        (_, "window") => "Isonomic quit unexpectedly".into(),
        (_, "engine") => "The Isonomic engine quit unexpectedly".into(),
        (_, id) => {
            let name = names.get(id).cloned();
            let name = name.unwrap_or_else(|| crate::apps::link_segment(id).to_string());
            format!("{name} stopped working")
        }
    }
}

/// What a person calls a bundled service.
fn service_name(name: &str) -> String {
    match name {
        "postgres" => "The database".into(),
        "valkey" => "The cache".into(),
        "" => "A background service".into(),
        other => format!("The {other} service"),
    }
}

/// What the notice says a report is for, and where it is.
pub const PURPOSE: &str = "A problem report helps fix what went wrong. It has no conversations, \
prompts or files, and you can see exactly what it holds.";
pub const KEPT_HERE: &str = "This report is kept on this Mac. Nothing leaves it.";
pub const ID_LABEL: &str = "Include my Isonomic ID so you can contact me about this report";

impl Shell {
    /// Show notices for kept problem reports, now and after any contained
    /// app failure, and the setting on Home.
    pub fn watch_reports(&mut self, kept: Arc<Kept>, cx: &mut Context<Self>) {
        let reports = cx.new(|cx| ProblemReports::new(kept, cx));
        reports.update(cx, |r, _| {
            for tab in self.tabs.iter().filter(|tab| tab.app.is_some()) {
                r.name_app(&tab.id, &tab.title);
            }
        });
        self.home
            .update(cx, |home, cx| home.show_report_setting(reports.clone(), cx));
        self.reports = Some(reports);
        self.reports_due(cx);
    }

    /// Look at the reports again after the engine's status changes: at
    /// once when a service is degraded, and again a moment later
    /// ([`ui::tokens::report_rescan_delay`]) after any change, because the
    /// engine writes its report after its own watch sees the failure.
    pub fn review_on_status(
        &mut self,
        mut live: tokio::sync::watch::Receiver<isonomic_client::Live>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |shell, cx| {
            while live.changed().await.is_ok() {
                let degraded = live.borrow_and_update().system.as_ref().is_some_and(|s| {
                    s.services
                        .iter()
                        .any(|svc| svc.state == isonomic_proto::ServiceState::Degraded)
                });
                let later = shell.clone();
                let looked = shell.update(cx, |shell, cx| {
                    if degraded {
                        shell.reports_due(cx);
                    }
                    cx.spawn(async move |_, cx| {
                        let delay = ui::tokens::report_rescan_delay();
                        cx.background_executor().timer(delay).await;
                        let _ = later.update(cx, |shell, cx| shell.reports_due(cx));
                    })
                    .detach();
                });
                if looked.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// An app reported a handled problem (`WindowRequest::Report`): write it
    /// as `app_error` with the app's id and version, then ask about it.
    /// Without reports (a self-check), it is dropped.
    pub fn app_reported(
        &mut self,
        app: &str,
        problem: isonomic_app::Problem,
        cx: &mut Context<Self>,
    ) {
        let Some(writer) = self
            .reports
            .as_ref()
            .and_then(|r| r.read(cx).writer.clone())
        else {
            return;
        };
        let version = self
            .app(app)
            .map(|tab| tab.read(cx).registration().manifest().version.clone())
            .unwrap_or_default();
        let message = match &problem.detail {
            Some(detail) => format!("{}: {detail}", problem.summary),
            None => problem.summary.clone(),
        };
        let component = reports::Component::App {
            id: app.to_string(),
            version,
        };
        let report = reports::Report::new(reports::Kind::AppError, component, message);
        cx.spawn(async move |shell, cx| {
            let saved = cx
                .background_spawn(async move { writer.save(report) })
                .await;
            if let Err(error) = saved {
                reports::log::line(&format!("app report not saved: {}", error.kind()));
            }
            let _ = shell.update(cx, |shell, cx| shell.reports_due(cx));
        })
        .detach();
    }

    /// The problem reports, once watched.
    pub fn reports(&self) -> Option<&Entity<ProblemReports>> {
        self.reports.as_ref()
    }

    pub(crate) fn reports_due(&mut self, cx: &mut Context<Self>) {
        self.reports_due = true;
        cx.notify();
    }

    /// Once `id` resolves, the notice offers to include the person's
    /// Isonomic ID. Without one (refused, or no engine) it never does.
    pub fn report_identity(
        &mut self,
        id: futures::future::BoxFuture<'static, Result<String, String>>,
        cx: &mut Context<Self>,
    ) {
        let Some(reports) = self.reports.clone() else {
            return;
        };
        cx.spawn(async move |_, cx| {
            if let Ok(id) = id.await {
                reports.update(cx, |r, cx| r.set_identity(Some(id), cx));
            }
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_engines_reports_have_plain_titles() {
        let names = HashMap::from([("com.isonomic.chat".to_string(), "Chat".to_string())]);
        let title = |doc: serde_json::Value| title_for(&doc, &names);
        let service = json!({ "kind": "service_failed", "component": { "id": "engine/postgres" },
                              "service": { "name": "postgres" } });
        assert_eq!(title(service), "The database stopped working");
        let model =
            json!({ "kind": "model_server_failed", "component": { "id": "engine/llama-server" } });
        assert_eq!(title(model), "The local model stopped working");
        let engine = json!({ "kind": "panic", "component": { "id": "engine" } });
        assert_eq!(title(engine), "The Isonomic engine quit unexpectedly");
        let app = json!({ "kind": "app_failed", "component": { "id": "com.isonomic.chat" } });
        assert_eq!(title(app), "Chat stopped working");
        let hang = json!({ "kind": "hang", "component": { "id": "window" } });
        assert_eq!(title(hang), "Isonomic stopped responding");
    }
}
