// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Headless tests of the logs view, in the real shell, against the SDK's
//! in-memory engine over the real client.
//!
//! The engine and client run on their own Tokio runtime, as the engine
//! link's do. GPUI's test scheduler refuses wakeups from other threads (it
//! would make the test nondeterministic), so the test relays between the
//! two on its own thread: it waits for the client to see the engine's
//! state, then publishes that to the window, and answers each log fetch
//! before handing it over.

extern crate ui as gpui;

use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use isonomic_client::{Client, Live};
use isonomic_proto::ServiceState;
use isonomic_testkit::TestEngine;
use shell::logs::{LogSource, LINES};
use shell::Shell;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
};

/// Fetches over the real client on the engine's runtime, then hands GPUI a
/// finished future.
struct Relayed {
    client: Client,
    runtime: Handle,
}

impl LogSource for Relayed {
    fn last_lines(&self, service: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        let lines = self
            .runtime
            .block_on(self.client.logs(service, Some(LINES)))
            .map_err(|error| error.to_string());
        Box::pin(async move { lines })
    }

    fn retry(&self, service: &str) -> BoxFuture<'static, Result<(), String>> {
        let sent = self
            .runtime
            .block_on(self.client.retry(service))
            .map_err(|error| error.to_string());
        Box::pin(async move { sent })
    }
}

struct Harness {
    engine: TestEngine,
    window: AnyWindowHandle,
    shell: Entity<Shell>,
    client_live: watch::Receiver<Live>,
    relay: watch::Sender<Live>,
    runtime: Runtime,
}

fn open(cx: &mut TestAppContext) -> Harness {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = TestEngine::new();
    engine.set_logs("postgres", &["ready to accept connections"]);
    let client = {
        let _inside = runtime.enter();
        engine.client()
    };
    let client_live = client.live();
    let (relay, live) = watch::channel(Live::default());
    let source = Relayed {
        client,
        runtime: runtime.handle().clone(),
    };
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(900.), px(600.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| Shell::new(live, Arc::new(source), cx))
        })
        .expect("open test window")
    });
    let harness = Harness {
        engine,
        window,
        shell,
        client_live,
        relay,
        runtime,
    };
    harness.settle(cx);
    harness
}

impl Harness {
    fn shown(&self, cx: &mut TestAppContext, id: &'static str) -> bool {
        cx.update_window(self.window, |_, window, cx| {
            window.render_frame(cx);
            window
                .try_find(id)
                .map(|element| element.visible())
                .unwrap_or(false)
        })
        .unwrap()
    }

    fn click(&self, cx: &mut TestAppContext, id: &'static str) {
        cx.update_window(self.window, |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
    }

    /// The lines the open logs view holds.
    fn lines(&self, cx: &mut TestAppContext) -> Option<Vec<String>> {
        self.shell.read_with(cx, |shell, cx| {
            let home = shell.home().read(cx);
            home.open_logs().map(|logs| logs.read(cx).lines().to_vec())
        })
    }

    /// Wait until the client has seen everything the engine did, then show
    /// the window what the client sees.
    fn settle(&self, cx: &mut TestAppContext) {
        let want = self.engine.system();
        let mut live = self.client_live.clone();
        let seen = self.runtime.block_on(async {
            let wait = live.wait_for(|live| live.system.as_ref() == Some(&want));
            tokio::time::timeout(Duration::from_secs(10), wait)
                .await
                .map(|seen| seen.map(|live| live.clone()))
        });
        let seen = seen
            .expect("the client caught up with the engine")
            .expect("the client is running");
        self.relay.send_replace(seen);
        cx.run_until_parked();
    }
}

#[ui::test]
fn a_service_opens_its_logs_and_they_follow_its_status(cx: &mut TestAppContext) {
    let h = open(cx);
    assert!(h.shown(cx, "service-postgres"));

    h.click(cx, "service-postgres");
    cx.run_until_parked();
    assert_eq!(
        h.lines(cx),
        Some(vec!["ready to accept connections".to_string()])
    );
    assert!(h.shown(cx, "logs-title-postgres"));
    assert!(h.shown(cx, "log-line-0"));
    assert!(!h.shown(cx, "service-postgres"), "the list is replaced");

    // New lines arrive with the next status event, not before.
    h.engine.set_logs(
        "postgres",
        &["ready to accept connections", "terminated by signal 9"],
    );
    cx.run_until_parked();
    assert_eq!(h.lines(cx).map(|lines| lines.len()), Some(1), "no polling");
    h.engine.set_state("postgres", ServiceState::Starting);
    h.settle(cx);
    assert_eq!(h.lines(cx).map(|lines| lines.len()), Some(2));
    assert!(h.shown(cx, "log-line-1"));

    h.click(cx, "logs-back");
    cx.run_until_parked();
    assert!(h.shown(cx, "service-postgres"));
    assert!(h.lines(cx).is_none());
}

#[ui::test]
fn a_degraded_service_says_so_above_its_last_lines(cx: &mut TestAppContext) {
    let h = open(cx);
    let kept: Vec<String> = (0..50).map(|n| format!("line {n}")).collect();
    let refs: Vec<&str> = kept.iter().map(String::as_str).collect();
    h.engine.set_logs("postgres", &refs);
    h.engine.set_state("postgres", ServiceState::Degraded);
    h.settle(cx);
    assert!(h.shown(cx, "state-degraded"));

    h.click(cx, "service-postgres");
    cx.run_until_parked();
    assert_eq!(h.lines(cx), Some(kept));
    assert!(h.shown(cx, "logs-degraded"));
    assert!(h.shown(cx, "log-line-0"));
}

#[ui::test]
fn retry_starts_a_degraded_service_again(cx: &mut TestAppContext) {
    let h = open(cx);
    h.engine.set_logs("postgres", &["too many restarts"]);
    h.engine.set_state("postgres", ServiceState::Degraded);
    h.engine.on_retry("postgres", ServiceState::Healthy);
    h.settle(cx);
    h.click(cx, "service-postgres");
    cx.run_until_parked();
    assert!(h.shown(cx, "logs-retry"));

    h.engine
        .set_logs("postgres", &["ready to accept connections"]);
    h.click(cx, "logs-retry");
    cx.run_until_parked();
    assert_eq!(h.engine.retries(), ["postgres"]);
    h.settle(cx);
    assert!(!h.shown(cx, "logs-retry"), "healthy again: no Retry");
    assert!(!h.shown(cx, "logs-degraded"));
    assert_eq!(
        h.lines(cx),
        Some(vec!["ready to accept connections".to_string()])
    );
}

#[ui::test]
fn a_refused_retry_says_why(cx: &mut TestAppContext) {
    let h = open(cx);
    h.engine.set_state("postgres", ServiceState::Healthy);
    h.settle(cx);
    // The view still believes the service is degraded; the engine knows
    // better and refuses.
    h.relay.send_modify(|live| {
        if let Some(system) = live.system.as_mut() {
            system.services[0].state = ServiceState::Degraded;
        }
    });
    cx.run_until_parked();
    h.click(cx, "service-postgres");
    cx.run_until_parked();
    h.click(cx, "logs-retry");
    cx.run_until_parked();
    assert!(h.shown(cx, "logs-retry-failed"));
}
