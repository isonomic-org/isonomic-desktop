//! Headless window tests: the real shell in a real (off-screen) window. The
//! engine link is replaced by the watch it would publish on, so each step is
//! deterministic.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::Live;
use isonomic_proto::{ServiceState, ServiceStatus, SystemStatus};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
};

/// No lines for any service: these tests are about status.
struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

fn open(cx: &mut TestAppContext, live: watch::Receiver<Live>) -> (AnyWindowHandle, Entity<Shell>) {
    cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(900.), px(600.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| Shell::new(live, Arc::new(NoLogs), cx))
        })
        .expect("open test window")
    })
}

/// What the engine link publishes for one database in `state`.
fn engine_reports(state: ServiceState, restarts: u32) -> Live {
    Live {
        connected: true,
        engine: Some("0.0.1".into()),
        system: Some(SystemStatus {
            aggregate: state,
            services: vec![ServiceStatus {
                name: "postgres".into(),
                tier: 1,
                state,
                restart_count: restarts,
                last_healthy_unix: Some(1_700_000_000),
            }],
            cold_start_ms: Some(180),
        }),
    }
}

/// Is `id` on screen, optionally inside the element `scope`?
fn shown(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    scope: Option<&'static str>,
    id: &'static str,
) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        let found = match scope {
            Some(scope) => window.within(scope).try_find(id),
            None => window.try_find(id),
        };
        found.map(|element| element.visible()).unwrap_or(false)
    })
    .unwrap()
}

#[ui::test]
fn the_window_shows_the_rail_and_follows_the_engine_untouched(cx: &mut TestAppContext) {
    let (publish, live) = watch::channel(Live::default());
    let (window, shell) = open(cx, live);

    // Before the engine answers: the rail is there, no services yet.
    assert!(shown(cx, window, None, "rail"));
    assert!(shown(cx, window, None, "rail-home"));
    assert!(shown(cx, window, None, "engine-connecting"));
    assert!(!shown(cx, window, None, "service-postgres"));
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.selected_tab().to_string()),
        "home"
    );

    // The database is reported healthy.
    publish
        .send(engine_reports(ServiceState::Healthy, 0))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, None, "aggregate-healthy"));
    assert!(shown(cx, window, Some("service-postgres"), "state-healthy"));
    assert!(!shown(cx, window, None, "engine-connecting"));

    // It is force-killed: the window shows it restarting, then healthy
    // again, with nobody touching the window.
    publish
        .send(engine_reports(ServiceState::Starting, 0))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, None, "aggregate-starting"));
    assert!(shown(
        cx,
        window,
        Some("service-postgres"),
        "state-starting"
    ));
    assert!(!shown(
        cx,
        window,
        Some("service-postgres"),
        "state-healthy"
    ));

    publish
        .send(engine_reports(ServiceState::Healthy, 1))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, None, "aggregate-healthy"));
    assert!(shown(cx, window, Some("service-postgres"), "state-healthy"));

    // The engine goes away: back to connecting, no stale services.
    publish.send(Live::default()).unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, None, "engine-connecting"));
    assert!(!shown(cx, window, None, "service-postgres"));
}

#[ui::test]
fn clicking_the_rail_keeps_the_selected_tab(cx: &mut TestAppContext) {
    let (_publish, live) = watch::channel(engine_reports(ServiceState::Healthy, 0));
    let (window, shell) = open(cx, live);
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("rail-home", cx);
    })
    .unwrap();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.selected_tab().to_string()),
        "home"
    );
    assert!(shown(cx, window, Some("service-postgres"), "state-healthy"));
}
