//! A service's logs: the last lines the supervisor kept for it, from
//! `system.logs`.
//!
//! The view fetches once when it opens and again whenever that service's
//! status changes (a new state or another restart), so it follows the same
//! events the Home tab does. It never polls, and the fetch runs on the
//! interface framework's executor without blocking the foreground thread.

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::{Client, Live};
use isonomic_proto::{ServiceState, ServiceStatus};
use tokio::sync::watch;
use ui::base::v_flex;
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

/// How many lines to ask for: what the supervisor keeps for a degraded
/// service.
pub const LINES: u32 = 50;

/// Where the lines come from. The real one is the engine client; tests can
/// script their own.
pub trait LogSource: Send + Sync + 'static {
    fn last_lines(&self, service: &str) -> BoxFuture<'static, Result<Vec<String>, String>>;
}

impl LogSource for Client {
    fn last_lines(&self, service: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        let client = self.clone();
        let service = service.to_string();
        Box::pin(async move {
            client
                .logs(&service, Some(LINES))
                .await
                .map_err(|error| error.to_string())
        })
    }
}

/// The part of a service's status that should bring new lines with it.
fn moment(status: &ServiceStatus) -> (ServiceState, u32) {
    (status.state, status.restart_count)
}

fn find(live: &Live, service: &str) -> Option<ServiceStatus> {
    live.system
        .as_ref()?
        .services
        .iter()
        .find(|status| status.name == service)
        .cloned()
}

pub struct LogsView {
    service: String,
    status: Option<ServiceStatus>,
    lines: Vec<String>,
    error: Option<String>,
    fetched: bool,
}

impl LogsView {
    pub fn new(
        service: String,
        source: Arc<dyn LogSource>,
        live: watch::Receiver<Live>,
        cx: &mut Context<Self>,
    ) -> Self {
        let status = find(&live.borrow(), &service);
        let name = service.clone();
        let mut seen = status.as_ref().map(moment);
        cx.spawn(async move |view, cx| {
            let mut live = live;
            loop {
                let outcome = source.last_lines(&name).await;
                let shown = view.update(cx, |view, cx| {
                    match outcome {
                        Ok(lines) => {
                            view.lines = lines;
                            view.error = None;
                        }
                        // Keep the last lines on screen; say why they are old.
                        Err(error) => view.error = Some(error),
                    }
                    view.fetched = true;
                    cx.notify();
                });
                if shown.is_err() {
                    return; // the view is gone
                }
                // Wait for this service's next change. A disconnect is not
                // one: the lines stay until the engine is back and reports.
                loop {
                    if live.changed().await.is_err() {
                        return;
                    }
                    let latest = find(&live.borrow_and_update(), &name);
                    let changed = latest.as_ref().map(moment);
                    let redrawn = view.update(cx, |view, cx| {
                        if latest.is_some() {
                            view.status = latest.clone();
                            cx.notify();
                        }
                    });
                    if redrawn.is_err() {
                        return;
                    }
                    if changed.is_some() && changed != seen {
                        seen = changed;
                        break;
                    }
                }
            }
        })
        .detach();
        Self {
            service,
            status,
            lines: Vec::new(),
            error: None,
            fetched: false,
        }
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    /// The lines on screen, oldest first.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    fn note(&self) -> Option<String> {
        if let Some(error) = &self.error {
            return Some(format!("Couldn't load new lines: {error}"));
        }
        if !self.fetched {
            return Some("Loading".to_string());
        }
        if self.lines.is_empty() {
            return Some("No lines yet".to_string());
        }
        match self.status.as_ref().map(|status| status.state) {
            Some(ServiceState::Degraded) => Some(format!(
                "Restarted too often. The last {} lines it wrote:",
                self.lines.len()
            )),
            _ => None,
        }
    }
}

impl Render for LogsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let degraded = matches!(
            self.status.as_ref().map(|status| status.state),
            Some(ServiceState::Degraded)
        );
        let lines: Vec<_> = self
            .lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                div()
                    .id(SharedString::from(format!("log-line-{index}")))
                    .test_support()
                    .font_family(theme.mono_font_family.clone())
                    .text_sm()
                    .child(SharedString::from(line.clone()))
            })
            .collect();
        let mut view = v_flex().id("logs").test_support().flex_1().gap_2().child(
            div()
                .id(SharedString::from(format!("logs-title-{}", self.service)))
                .test_support()
                .text_lg()
                .child(SharedString::from(format!("{} logs", self.service))),
        );
        if let Some(note) = self.note() {
            let id = if degraded {
                "logs-degraded"
            } else {
                "logs-note"
            };
            view = view.child(
                div()
                    .id(id)
                    .test_support()
                    .text_color(if degraded {
                        theme.danger
                    } else {
                        theme.muted_foreground
                    })
                    .child(SharedString::from(note)),
            );
        }
        view.child(
            v_flex()
                .id("log-lines")
                .flex_1()
                .overflow_y_scroll()
                .children(lines),
        )
    }
}
