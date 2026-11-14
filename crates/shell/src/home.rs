//! The Home tab: every service's live state.
//!
//! The tab holds the latest `Live` value and redraws when the engine link's
//! watch changes. It never polls and never calls the engine while drawing.

use isonomic_client::Live;
use isonomic_proto::{ServiceState, ServiceStatus};
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, tokens, App, Context, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, TestSupportExt as _, Window,
};

pub struct HomeTab {
    live: Live,
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

fn headline(live: &Live) -> &'static str {
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
    pub fn new(live: watch::Receiver<Live>, cx: &mut Context<Self>) -> Self {
        let current = live.borrow().clone();
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
        Self { live: current }
    }

    /// The value currently drawn.
    pub fn live(&self) -> &Live {
        &self.live
    }
}

impl Render for HomeTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                h_flex()
                    .id(SharedString::from(format!("service-{}", service.name)))
                    .test_support()
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
