//! Isonomic Chat mounted in the real shell, headless, against the SDK
//! testkit: it is on the rail and opens into its own view.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_app::testing::wait_until;
use isonomic_app::{register, App as _, MemorySettings};
use isonomic_client::Live;
use isonomic_testkit::TestEngine;
use shell::apps::{AppHost, AppState};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{px, size, AppContext as _, Bounds, Point, TestAppContext, WindowBounds, WindowOptions};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[ui::test]
fn chat_mounts_and_opens(cx: &mut TestAppContext) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let host = AppHost::new(
        engine.client_on(runtime.handle()),
        Arc::new(MemorySettings::default()),
    );
    let id = isonomic_chat::Chat::manifest().id;
    assert_eq!(id, "com.isonomic.chat");

    let (_, live) = watch::channel(Live::default());
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1100.), px(720.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.mount(register::<isonomic_chat::Chat>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    });

    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("rail-com.isonomic.chat", cx);
    })
    .unwrap();
    let tab = shell.read_with(cx, |s, _| s.app(&id).cloned()).unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    let drawn = cx
        .update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window
                .try_find("app-com.isonomic.chat")
                .map(|e| e.visible())
        })
        .unwrap();
    assert_eq!(drawn, Some(true));
}

#[ui::test]
fn a_draft_and_enter_start_a_chat(cx: &mut TestAppContext) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let host = AppHost::new(
        engine.client_on(runtime.handle()),
        Arc::new(MemorySettings::default()),
    );
    let (_, live) = watch::channel(Live::default());
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions::default();
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.mount(register::<isonomic_chat::Chat>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    });
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("rail-com.isonomic.chat", cx);
    })
    .unwrap();
    let tab = shell
        .read_with(cx, |s, _| s.app("com.isonomic.chat").cloned())
        .unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    let chat = tab.read_with(cx, |tab, _| match tab.state() {
        AppState::Open(app) => app.view().downcast::<isonomic_chat::Chat>().ok(),
        _ => None,
    });
    let chat_shell = chat.unwrap().read_with(cx, |chat, _| chat.shell().clone());
    cx.update_window(window, |_, window, cx| {
        chat_shell.update(cx, |s, cx| {
            s.set_draft("Plan one day in Lisbon", window, cx)
        });
        window.render_frame(cx);
        let handled = window.dispatch_keystroke(ui::Keystroke::parse("enter").unwrap(), cx);
        eprintln!("enter handled: {handled}");
    })
    .unwrap();
    cx.run_until_parked();
    let route = chat_shell.read_with(cx, |s, _| s.route().clone());
    eprintln!("route after enter: {route:?}");
}
