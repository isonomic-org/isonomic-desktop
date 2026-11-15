//! The app host in a headless window: the SDK's smallest app crate mounted
//! as a tab, against the in-memory engine. Nothing here waits on a real
//! engine.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use hello_app::{Hello, Phase};
use isonomic_app::testing::wait_until;
use isonomic_app::{register, App as _, Attention, MemorySettings};
use isonomic_client::Live;
use isonomic_proto::auth::Interface;
use isonomic_proto::{ErrorCode, WireError};
use isonomic_testkit::TestEngine;
use shell::apps::{AppHost, AppState};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
};

const HELLO: &str = "com.isonomic.hello";

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// An in-memory engine with a shell client on its own runtime, as the
/// engine link gives the window.
struct Engine {
    engine: TestEngine,
    host: AppHost,
    _runtime: tokio::runtime::Runtime,
}

fn engine() -> Engine {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let client = engine.client_on(runtime.handle());
    let host = AppHost::new(client, Arc::new(MemorySettings::default()));
    Engine {
        engine,
        host,
        _runtime: runtime,
    }
}

fn open(cx: &mut TestAppContext, host: AppHost) -> (AnyWindowHandle, Entity<Shell>) {
    let (_, live) = watch::channel(Live::default());
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
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.mount(register::<Hello>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    })
}

fn shown(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id).map(|e| e.visible()).unwrap_or(false)
    })
    .unwrap()
}

fn click(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click(id, cx);
    })
    .unwrap();
}

fn grants(engine: &Engine) -> usize {
    let calls = engine.engine.calls();
    calls.iter().filter(|c| c.method == "apps.grant").count()
}

#[ui::test]
fn an_app_crate_mounts_as_a_tab_and_opens_on_first_click(cx: &mut TestAppContext) {
    let engine = engine();
    let (window, shell) = open(cx, engine.host.clone());

    // On the rail, but not built until it is opened: nothing granted yet.
    assert!(shown(cx, window, "rail-com.isonomic.hello"));
    assert!(!shown(cx, window, "app-com.isonomic.hello"));
    assert_eq!(grants(&engine), 0);

    click(cx, window, "rail-com.isonomic.hello");
    assert_eq!(
        shell.read_with(cx, |s, _| s.selected_tab().to_string()),
        HELLO
    );
    let tab = shell.read_with(cx, |s, _| s.app(HELLO).cloned()).unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    let hello = tab.read_with(cx, |tab, _| match tab.state() {
        AppState::Open(app) => app.view().downcast::<Hello>().ok(),
        _ => None,
    });
    let hello = hello.expect("the tab draws Hello's root view");
    wait_until(cx, &hello, |hello| *hello.phase() == Phase::Done);
    assert!(shown(cx, window, "app-com.isonomic.hello"));

    // The shell granted the token from the app's manifest; every call after
    // that went out under it.
    assert_eq!(grants(&engine), 1);
    let calls = engine.engine.calls();
    let mine: Vec<&str> = calls
        .iter()
        .filter(|c| c.app.as_deref() == Some(HELLO))
        .map(|c| c.method.as_str())
        .collect();
    assert_eq!(mine, ["models.list", "models.stream"]);

    // Hello asked for attention when it finished: the rail shows it.
    wait_until(cx, &tab, |tab| tab.attention() == Attention::Dot);
    assert!(shown(cx, window, "attention-com.isonomic.hello"));

    // Back to Home and again: the app is not built twice.
    click(cx, window, "rail-home");
    click(cx, window, "rail-com.isonomic.hello");
    cx.run_until_parked();
    assert_eq!(grants(&engine), 1);
}

#[ui::test]
fn a_refused_grant_shows_and_is_tried_again(cx: &mut TestAppContext) {
    let engine = engine();
    let refused = WireError::new(ErrorCode::CapabilityDenied, "not approved");
    engine.engine.fail(Interface::Apps, refused);
    let (window, shell) = open(cx, engine.host.clone());

    click(cx, window, "rail-com.isonomic.hello");
    let tab = shell.read_with(cx, |s, _| s.app(HELLO).cloned()).unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Failed(_)));
    assert!(shown(cx, window, "app-failed"));

    engine.engine.recover(Interface::Apps);
    click(cx, window, "rail-home");
    click(cx, window, "rail-com.isonomic.hello");
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    assert_eq!(Hello::manifest().id, HELLO);
}

#[test]
fn a_link_names_its_app_by_the_last_part_of_the_id() {
    use shell::apps::{link_segment, link_target};
    assert_eq!(link_segment("com.isonomic.chat"), "chat");
    assert_eq!(link_target("isonomic://chat/threads/t-1"), Some("chat"));
    assert_eq!(link_target("isonomic://hello?x=1"), Some("hello"));
    assert_eq!(link_target("isonomic:///x"), None);
    assert_eq!(link_target("https://chat/x"), None);
}

#[ui::test]
fn a_deep_link_opens_its_app_and_reaches_it_once_built(cx: &mut TestAppContext) {
    let engine = engine();
    let (window, shell) = open(cx, engine.host.clone());
    let link = "isonomic://hello/models/chat-small";

    // Not built yet: the link selects the tab, builds it, then arrives.
    let handled = cx
        .update_window(window, |_, window, cx| {
            shell.update(cx, |shell, cx| shell.open_link(link, window, cx))
        })
        .unwrap();
    assert!(handled);
    assert_eq!(
        shell.read_with(cx, |s, _| s.selected_tab().to_string()),
        HELLO
    );
    let tab = shell.read_with(cx, |s, _| s.app(HELLO).cloned()).unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    let hello = tab.read_with(cx, |tab, _| match tab.state() {
        AppState::Open(app) => app.view().downcast::<Hello>().ok(),
        _ => None,
    });
    let hello = hello.unwrap();
    assert_eq!(hello.read_with(cx, |h, _| h.opened().to_vec()), [link]);

    // A link for an app that isn't mounted is not handled.
    let other = cx
        .update_window(window, |_, window, cx| {
            shell.update(cx, |shell, cx| {
                shell.open_link("isonomic://notes/x", window, cx)
            })
        })
        .unwrap();
    assert!(!other);

    // Search reaches loaded apps, and each hit names its app.
    wait_until(cx, &hello, |hello| *hello.phase() == Phase::Done);
    let hits = shell.update(cx, |shell, cx| shell.search("chat", cx));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].0.as_ref(), HELLO);
    assert_eq!(hits[0].1.link, "isonomic://hello/models/chat-small");
}
