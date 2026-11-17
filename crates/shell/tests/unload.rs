// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Unloading app tabs past the hardware profile's cap, in a headless window
//! against the SDK testkit. Three tiny apps stand in for first-party ones:
//! each draws its name and counts its unloads.

extern crate ui as gpui;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_app::{register, App, AppContext, Capabilities, Manifest, MemorySettings};
use isonomic_client::Live;
use isonomic_testkit::TestEngine;
use shell::apps::{loaded_cap, AppHost};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    div, px, size, AnyView, AnyWindowHandle, AppContext as _, Bounds, Context, Entity, IntoElement,
    ParentElement as _, Point, Render, TestAppContext, Window, WindowBounds, WindowOptions,
};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

macro_rules! probe {
    ($name:ident, $id:literal, $unloads:ident) => {
        static $unloads: AtomicUsize = AtomicUsize::new(0);
        struct $name;
        impl App for $name {
            fn manifest() -> Manifest {
                Manifest {
                    id: $id.into(),
                    name: stringify!($name).into(),
                    version: "0.1.0".into(),
                    icon: "dot".into(),
                    capabilities: Capabilities::default(),
                    schemas: vec![],
                }
            }
            fn build(_: AppContext, _: &mut Window, app: &mut ui::App) -> AnyView {
                app.new(|_| $name).into()
            }
            fn on_unload(&mut self) {
                $unloads.fetch_add(1, Ordering::SeqCst);
            }
        }
        impl Render for $name {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().child(stringify!($name))
            }
        }
    };
}

probe!(Alpha, "com.isonomic.test-alpha", ALPHA_UNLOADS);
probe!(Beta, "com.isonomic.test-beta", BETA_UNLOADS);
probe!(Gamma, "com.isonomic.test-gamma", GAMMA_UNLOADS);

const ALPHA: &str = "com.isonomic.test-alpha";
const BETA: &str = "com.isonomic.test-beta";
const GAMMA: &str = "com.isonomic.test-gamma";

struct Harness {
    engine: TestEngine,
    window: AnyWindowHandle,
    shell: Entity<Shell>,
    _runtime: tokio::runtime::Runtime,
}

fn open(cx: &mut TestAppContext) -> Harness {
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
                shell.mount(register::<Alpha>(), host.clone(), cx);
                shell.mount(register::<Beta>(), host.clone(), cx);
                shell.mount(register::<Gamma>(), host.clone(), cx);
                shell
            })
        })
        .expect("open test window")
    });
    Harness {
        engine,
        window,
        shell,
        _runtime: runtime,
    }
}

impl Harness {
    /// Open the tab of `id` and wait until its app is built.
    fn open_app(&self, cx: &mut TestAppContext, id: &str) {
        let rail = ui::SharedString::from(format!("rail-{id}"));
        cx.update_window(self.window, |_, window, cx| {
            window.render_frame(cx);
            window.click(rail.clone(), cx);
        })
        .unwrap();
        let tab = self.shell.read_with(cx, |s, _| s.app(id).cloned()).unwrap();
        isonomic_app::testing::wait_until(cx, &tab, |tab| {
            matches!(tab.state(), shell::apps::AppState::Open(_))
        });
    }

    fn loaded(&self, cx: &mut TestAppContext, id: &str) -> bool {
        let tab = self.shell.read_with(cx, |s, _| s.app(id).cloned()).unwrap();
        tab.read_with(cx, |tab, _| tab.is_loaded())
    }

    fn grants(&self) -> usize {
        let calls = self.engine.calls();
        calls.iter().filter(|c| c.method == "apps.grant").count()
    }
}

#[test]
fn the_cap_follows_the_supervisor_spec() {
    assert_eq!(loaded_cap("low"), Some(2));
    assert_eq!(loaded_cap("standard"), Some(4));
    assert_eq!(loaded_cap("high"), None);
}

#[ui::test]
fn past_the_cap_the_least_recently_opened_app_is_unloaded(cx: &mut TestAppContext) {
    let h = open(cx);
    h.shell.update(cx, |shell, cx| {
        let low: BoxFuture<'static, Result<String, String>> =
            Box::pin(async { Ok("low".to_string()) });
        shell.follow_profile(low, cx)
    });
    cx.run_until_parked();

    h.open_app(cx, ALPHA);
    h.open_app(cx, BETA);
    assert!(h.loaded(cx, ALPHA) && h.loaded(cx, BETA));
    assert_eq!(ALPHA_UNLOADS.load(Ordering::SeqCst), 0);

    // A third app: Alpha, opened longest ago, is told and unloaded.
    h.open_app(cx, GAMMA);
    assert!(!h.loaded(cx, ALPHA));
    assert!(h.loaded(cx, BETA) && h.loaded(cx, GAMMA));
    assert_eq!(ALPHA_UNLOADS.load(Ordering::SeqCst), 1);

    // Reopening Alpha builds it again without a new grant, and Beta goes.
    h.open_app(cx, ALPHA);
    assert!(!h.loaded(cx, BETA));
    assert!(h.loaded(cx, ALPHA) && h.loaded(cx, GAMMA));
    assert_eq!(BETA_UNLOADS.load(Ordering::SeqCst), 1);
    assert_eq!(GAMMA_UNLOADS.load(Ordering::SeqCst), 0);
    assert_eq!(h.grants(), 3);
}
