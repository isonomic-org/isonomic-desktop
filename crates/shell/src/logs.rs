//! A service's logs: the last lines the supervisor kept for it, from
//! `system.logs`.
//!
//! The view fetches once when it opens and again whenever that service's
//! status changes (a new state or another restart), so it follows the same
//! events the Home tab does. It never polls, and the fetch runs on the
//! interface framework's executor without blocking the foreground thread.
//!
//! A degraded service gets a Retry button (`system.retry`). The engine
//! answers at once; the service's next status events bring the outcome and
//! its new lines.

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::{Client, Live};
use isonomic_proto::{ServiceState, ServiceStatus};
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::alert::Alert;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::theme::ActiveTheme as _;
use ui::component::Disableable as _;
use ui::component::{Icon, IconName};
use ui::{
    div, Context, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

/// How many lines to ask for: what the supervisor keeps for a degraded
/// service.
pub const LINES: u32 = 50;

/// Where the lines come from, and who restarts a degraded service. The
/// real one is the engine client; tests can script their own.
pub trait LogSource: Send + Sync + 'static {
    fn last_lines(&self, service: &str) -> BoxFuture<'static, Result<Vec<String>, String>>;

    /// Ask the engine to start a degraded service again.
    fn retry(&self, service: &str) -> BoxFuture<'static, Result<(), String>>;
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

    fn retry(&self, service: &str) -> BoxFuture<'static, Result<(), String>> {
        let client = self.clone();
        let service = service.to_string();
        Box::pin(async move {
            client
                .retry(&service)
                .await
                .map_err(|error| error.to_string())
        })
    }
}

/// Where a retry is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Retry {
    Idle,
    /// Sent; waiting for the engine to accept it.
    Sending,
    /// The engine refused it or could not be reached.
    Failed(String),
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
    source: Arc<dyn LogSource>,
    retry: Retry,
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
        let kept = Arc::clone(&source);
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
            source: kept,
            retry: Retry::Idle,
            status,
            lines: Vec::new(),
            error: None,
            fetched: false,
        }
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn retry_state(&self) -> &Retry {
        &self.retry
    }

    fn retry(&mut self, cx: &mut Context<Self>) {
        if self.retry == Retry::Sending {
            return;
        }
        self.retry = Retry::Sending;
        cx.notify();
        let sent = self.source.retry(&self.service);
        cx.spawn(async move |view, cx| {
            let outcome = sent.await;
            let _ = view.update(cx, |view, cx| {
                view.retry = match outcome {
                    Ok(()) => Retry::Idle,
                    Err(error) => Retry::Failed(error),
                };
                cx.notify();
            });
        })
        .detach();
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
            view = view.child(if degraded {
                div().id("logs-degraded").test_support().child(
                    Alert::error("logs-degraded-alert", note)
                        .title(format!("{} stopped restarting", self.service)),
                )
            } else {
                div()
                    .id("logs-note")
                    .test_support()
                    .text_color(theme.muted_foreground)
                    .child(SharedString::from(note))
            });
        }
        if degraded {
            let sending = self.retry == Retry::Sending;
            let mut row = h_flex().gap_2().child(
                Button::new("logs-retry")
                    .primary()
                    .icon(Icon::from(IconName::RefreshCw))
                    .label(if sending { "Retrying" } else { "Retry" })
                    .loading(sending)
                    .disabled(sending)
                    .on_click(cx.listener(|view, _, _, cx| view.retry(cx))),
            );
            if let Retry::Failed(error) = &self.retry {
                row = row.child(
                    div()
                        .id("logs-retry-failed")
                        .test_support()
                        .text_color(theme.danger)
                        .child(SharedString::from(format!("Couldn't retry: {error}"))),
                );
            }
            view = view.child(row);
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
