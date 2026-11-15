//! The Home tab: every service's live state. Clicking a service opens its
//! logs; Back returns to the list.
//!
//! The tab holds the latest `Live` value and redraws when the engine link's
//! watch changes. It never polls and never calls the engine while drawing.

use std::sync::Arc;

use isonomic_client::Live;
use isonomic_proto::agents::SchemaVersion;
use isonomic_proto::{ServiceState, ServiceStatus};
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::button::Button;
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, tokens, App, AppContext as _, Context, Entity, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, Window,
};

use crate::logs::{LogSource, LogsView};

pub struct HomeTab {
    live: Live,
    updates: watch::Receiver<Live>,
    source: Arc<dyn LogSource>,
    logs: Option<Entity<LogsView>>,
    /// Why the engine can't be reached, from the engine link.
    problem: Option<String>,
    /// Something the person should do, shown until the next launch.
    notice: Option<String>,
}

/// Wire name of a state, used in element ids.
fn state_id(state: ServiceState) -> &'static str {
    match state {
        ServiceState::Stopped => "stopped",
        ServiceState::Starting => "starting",
        ServiceState::Healthy => "healthy",
        ServiceState::Degraded => "degraded",
    }
}

/// What a person reads for a state. A service that is starting again after
/// having been healthy is restarting.
fn state_label(status: &ServiceStatus) -> &'static str {
    match status.state {
        ServiceState::Stopped => "Stopped",
        ServiceState::Starting if status.last_healthy_unix.is_some() => "Restarting",
        ServiceState::Starting => "Starting",
        ServiceState::Healthy => "Healthy",
        ServiceState::Degraded => "Degraded",
    }
}

/// A warning when the engine writes agent sessions in a schema version this
/// window can't read. Apps would show such sessions generically or not at
/// all, so the person is told to update.
pub fn schema_warning(live: &Live) -> Option<String> {
    let schema = live.schema?;
    if schema.readable() {
        return None;
    }
    Some(format!(
        "The engine writes conversations in a newer format (version {}) than this window reads (version {}). Update Isonomic to see them.",
        schema.version,
        SchemaVersion::CURRENT.version
    ))
}

/// One line for the whole engine, shared with the menu bar.
pub fn headline(live: &Live) -> &'static str {
    match live.system.as_ref().map(|system| system.aggregate) {
        None => "Connecting to the engine",
        Some(ServiceState::Healthy) => "All services healthy",
        Some(ServiceState::Starting) => "Services starting",
        Some(ServiceState::Degraded) => "A service needs attention",
        Some(ServiceState::Stopped) => "Services stopped",
    }
}

fn state_color(state: ServiceState, cx: &App) -> Hsla {
    let theme = cx.theme();
    match state {
        ServiceState::Healthy => theme.success,
        ServiceState::Starting => theme.warning,
        ServiceState::Degraded => theme.danger,
        ServiceState::Stopped => theme.muted_foreground,
    }
}

impl HomeTab {
    pub fn new(
        live: watch::Receiver<Live>,
        source: Arc<dyn LogSource>,
        cx: &mut Context<Self>,
    ) -> Self {
        let current = live.borrow().clone();
        let kept = live.clone();
        let mut updates = live;
        cx.spawn(async move |tab, cx| {
            // Wakes only when the engine link publishes a change.
            while updates.changed().await.is_ok() {
                let latest = updates.borrow_and_update().clone();
                let redrawn = tab.update(cx, |tab, cx| {
                    tab.live = latest;
                    cx.notify();
                });
                if redrawn.is_err() {
                    break; // the tab is gone
                }
            }
        })
        .detach();
        Self {
            live: current,
            updates: kept,
            source,
            logs: None,
            problem: None,
            notice: None,
        }
    }

    /// Show `notice` at the top of Home, such as "move the app".
    pub fn set_notice(&mut self, notice: String, cx: &mut Context<Self>) {
        self.notice = Some(notice);
        cx.notify();
    }

    /// Follow the engine link's problems: shown while not connected.
    pub fn follow_problems(
        &mut self,
        mut problems: watch::Receiver<Option<String>>,
        cx: &mut Context<Self>,
    ) {
        self.problem = problems.borrow_and_update().clone();
        cx.spawn(async move |tab, cx| {
            while problems.changed().await.is_ok() {
                let latest = problems.borrow_and_update().clone();
                let redrawn = tab.update(cx, |tab, cx| {
                    tab.problem = latest;
                    cx.notify();
                });
                if redrawn.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    /// The problem on screen, if any.
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref().filter(|_| !self.live.connected)
    }

    /// The value currently drawn.
    pub fn live(&self) -> &Live {
        &self.live
    }

    /// The service whose logs are open, if any.
    pub fn open_logs(&self) -> Option<&Entity<LogsView>> {
        self.logs.as_ref()
    }

    fn show_logs(&mut self, service: String, cx: &mut Context<Self>) {
        let source = Arc::clone(&self.source);
        let live = self.updates.clone();
        self.logs = Some(cx.new(|cx| LogsView::new(service, source, live, cx)));
        cx.notify();
    }

    fn close_logs(&mut self, cx: &mut Context<Self>) {
        self.logs = None;
        cx.notify();
    }
}

impl Render for HomeTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(logs) = &self.logs {
            return v_flex()
                .id("home")
                .size_full()
                .p_6()
                .gap_4()
                .child(
                    h_flex().child(
                        Button::new("logs-back")
                            .label("Back")
                            .on_click(cx.listener(|tab, _, _, cx| tab.close_logs(cx))),
                    ),
                )
                .child(logs.clone());
        }
        let muted = cx.theme().muted_foreground;
        let border = cx.theme().border;
        let aggregate = self.live.system.as_ref().map(|system| system.aggregate);
        let headline_id = match aggregate {
            Some(state) => format!("aggregate-{}", state_id(state)),
            None => "engine-connecting".to_string(),
        };
        let services: Vec<ServiceStatus> = self
            .live
            .system
            .as_ref()
            .map(|system| system.services.clone())
            .unwrap_or_default();

        let rows: Vec<_> = services
            .iter()
            .map(|service| {
                let restarts: SharedString = match service.restart_count {
                    0 => "".into(),
                    1 => "1 restart".into(),
                    n => format!("{n} restarts").into(),
                };
                let name = service.name.clone();
                h_flex()
                    .id(SharedString::from(format!("service-{}", service.name)))
                    .test_support()
                    .cursor_pointer()
                    .on_click(cx.listener(move |tab, _, _, cx| tab.show_logs(name.clone(), cx)))
                    .gap_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "state-{}",
                                state_id(service.state)
                            )))
                            .test_support()
                            .size(tokens::status_dot())
                            .rounded_full()
                            .bg(state_color(service.state, cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(SharedString::from(service.name.clone())),
                    )
                    .child(div().text_color(muted).child(restarts))
                    .child(div().text_color(muted).child(state_label(service)))
            })
            .collect();

        v_flex()
            .id("home")
            .size_full()
            .p_6()
            .gap_4()
            .child(
                div()
                    .id(SharedString::from(headline_id))
                    .test_support()
                    .text_lg()
                    .child(headline(&self.live)),
            )
            .children(self.notice.clone().map(|notice| {
                div()
                    .id("home-notice")
                    .test_support()
                    .text_color(cx.theme().warning)
                    .child(SharedString::from(notice))
            }))
            .children(self.problem().map(|problem| {
                div()
                    .id("engine-problem")
                    .test_support()
                    .text_color(cx.theme().danger)
                    .child(SharedString::from(problem.to_string()))
            }))
            .children(schema_warning(&self.live).map(|warning| {
                div()
                    .id("schema-unreadable")
                    .test_support()
                    .text_color(cx.theme().danger)
                    .child(SharedString::from(warning))
            }))
            .child(v_flex().id("services").children(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(state: ServiceState, last_healthy: Option<i64>) -> ServiceStatus {
        ServiceStatus {
            name: "postgres".into(),
            tier: 1,
            state,
            restart_count: 0,
            last_healthy_unix: last_healthy,
        }
    }

    #[test]
    fn a_newer_session_schema_is_flagged_and_the_same_or_older_is_not() {
        let at = |version, revision| Live {
            schema: Some(SchemaVersion { version, revision }),
            ..Live::default()
        };
        let current = SchemaVersion::CURRENT;
        assert_eq!(schema_warning(&Live::default()), None);
        assert_eq!(
            schema_warning(&at(current.version, current.revision + 1)),
            None
        );
        assert_eq!(
            schema_warning(&at(current.version.saturating_sub(1), 0)),
            None
        );
        let warning = schema_warning(&at(current.version + 1, 0)).unwrap();
        assert!(warning.contains(&format!("version {}", current.version + 1)));
    }

    #[test]
    fn starting_again_after_being_healthy_reads_as_restarting() {
        assert_eq!(
            state_label(&status(ServiceState::Starting, None)),
            "Starting"
        );
        assert_eq!(
            state_label(&status(ServiceState::Starting, Some(1))),
            "Restarting"
        );
        assert_eq!(
            state_label(&status(ServiceState::Healthy, Some(1))),
            "Healthy"
        );
        assert_eq!(
            state_label(&status(ServiceState::Degraded, Some(1))),
            "Degraded"
        );
    }

    #[test]
    fn the_headline_says_connecting_until_a_snapshot_arrives() {
        assert_eq!(headline(&Live::default()), "Connecting to the engine");
    }
}
