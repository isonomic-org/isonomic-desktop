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
        schema: None,
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

#[ui::test]
fn a_problem_reaching_the_engine_shows_until_it_connects(cx: &mut TestAppContext) {
    let (publish, live) = watch::channel(Live::default());
    let (window, shell) = open(cx, live);
    let (report, problems) = watch::channel(None::<String>);
    shell.update(cx, |shell, cx| {
        shell
            .home()
            .update(cx, |home, cx| home.follow_problems(problems, cx))
    });
    assert!(!shown(cx, window, None, "engine-problem"));

    report
        .send(Some(
            "The engine stopped as it started (exit code 1).".into(),
        ))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, None, "engine-connecting"));
    assert!(shown(cx, window, None, "engine-problem"));

    // Connected: the problem is old news, even before the link clears it.
    publish
        .send(engine_reports(ServiceState::Healthy, 0))
        .unwrap();
    cx.run_until_parked();
    assert!(!shown(cx, window, None, "engine-problem"));
}

#[ui::test]
fn a_notice_asks_to_move_the_app_in_plain_words(cx: &mut TestAppContext) {
    let (_publish, live) = watch::channel(Live::default());
    let (window, shell) = open(cx, live);
    assert!(!shown(cx, window, None, "home-notice"));
    shell.update(cx, |shell, cx| {
        shell.home().update(cx, |home, cx| {
            home.set_notice(
                "Move Isonomic to the Applications folder and open it from there.".into(),
                cx,
            )
        })
    });
    cx.run_until_parked();
    assert!(shown(cx, window, None, "home-notice"));
}

/// The headless test window can't read a title back (it keeps none), so
/// this checks what the app opens its window with; the title bar itself is
/// checked on the Mac.
#[ui::test]
fn the_window_opens_titled(cx: &mut TestAppContext) {
    let options = cx.update(|cx| shell::lifecycle::window_options(cx));
    let title = options.titlebar.and_then(|bar| bar.title);
    assert_eq!(title.as_deref(), Some(shell::lifecycle::TITLE));
    assert!(options.window_bounds.is_some(), "it opens at a set size");
}
