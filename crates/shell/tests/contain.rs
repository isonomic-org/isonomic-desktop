//! An app that panics in a call the window makes is contained: its tab
//! says it stopped working, and the window and its other tabs go on.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_app::{register, App, AppContext, Capabilities, Manifest, MemorySettings};
use isonomic_client::Live;
use isonomic_testkit::TestEngine;
use shell::apps::{AppHost, AppState};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{AnyView, AppContext as _, TestAppContext, Window, WindowOptions};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

struct Broken;

impl App for Broken {
    fn manifest() -> Manifest {
        Manifest {
            id: "com.isonomic.test-broken".into(),
            name: "Broken".into(),
            version: "0.1.0".into(),
            icon: "cpu".into(),
            capabilities: Capabilities::default(),
            schemas: vec![],
        }
    }

    fn build(_: AppContext, _: &mut Window, _: &mut ui::App) -> AnyView {
        panic!("the app could not build its view")
    }
}

#[ui::test]
fn an_app_that_panics_while_building_is_contained(cx: &mut TestAppContext) {
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
        ui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.mount(register::<Broken>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    });
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("rail-com.isonomic.test-broken", cx);
    })
    .unwrap();
    let tab = shell
        .read_with(cx, |s, _| s.app("com.isonomic.test-broken").cloned())
        .unwrap();
    isonomic_app::testing::wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Failed(_)));
    let failed = tab.read_with(cx, |tab, _| match tab.state() {
        AppState::Failed(error) => error.clone(),
        _ => String::new(),
    });
    assert!(failed.contains("could not build its view"), "{failed}");
    // The window is still there: its rail draws and Home still opens.
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        assert!(window.try_find("app-failed").is_some_and(|e| e.visible()));
        window.click("rail-home", cx);
    })
    .unwrap();
    assert_eq!(
        shell.read_with(cx, |s, _| s.selected_tab().to_string()),
        "home"
    );
}
