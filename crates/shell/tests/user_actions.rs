// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! An app's ask for the person's user-action token, in a headless window:
//! answered by the shell's client only from the app's own click handler,
//! refused anywhere else.

extern crate ui as gpui;

use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use isonomic_app::testing::wait_until;
use isonomic_app::{register, App, AppContext, Manifest, MemorySettings};
use isonomic_client::Live;
use isonomic_proto::agents::{ApprovalId, SessionId};
use isonomic_proto::Capabilities;
use isonomic_testkit::TestEngine;
use shell::apps::{AppHost, AppState};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    div, px, size, AnyView, AnyWindowHandle, AppContext as _, Bounds, Context, Entity,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render,
    StatefulInteractiveElement as _, Styled as _, TestAppContext, TestSupportExt as _, Window,
    WindowBounds, WindowOptions,
};

const ASKER: &str = "com.isonomic.asker";

/// What each ask came back with: the refusal's code, or "token".
static ANSWERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// An app with one Allow button that asks for the token as it is clicked.
struct Asker {
    app: AppContext,
}

impl Asker {
    /// Ask now: from the click handler, or from anywhere else.
    fn ask(&self, cx: &mut Context<Self>) {
        let asked = self
            .app
            .user_action(&SessionId::from("s-1"), &ApprovalId::from("a-1"));
        cx.background_spawn(async move {
            let answer = match asked.await {
                Ok(_) => "token".to_string(),
                Err(error) => error.code().to_string(),
            };
            ANSWERS.lock().unwrap().push(answer);
        })
        .detach();
    }
}

impl Render for Asker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            div()
                .id("allow")
                .test_support()
                .size(px(80.))
                .child("Allow")
                .on_click(cx.listener(|this, _, _, cx| this.ask(cx))),
        )
    }
}

impl App for Asker {
    fn manifest() -> Manifest {
        Manifest {
            id: ASKER.into(),
            name: "Asker".into(),
            version: "0.1.0".into(),
            icon: "sparkle".into(),
            capabilities: Capabilities::default(),
            schemas: vec![],
        }
    }

    fn build(app: AppContext, _: &mut Window, cx: &mut ui::App) -> AnyView {
        cx.new(|_| Asker { app }).into()
    }
}

fn answers(cx: &mut TestAppContext, count: usize) -> Vec<String> {
    for _ in 0..200 {
        cx.run_until_parked();
        if ANSWERS.lock().unwrap().len() >= count {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    ANSWERS.lock().unwrap().clone()
}

#[ui::test]
fn a_token_is_given_only_inside_the_persons_click(cx: &mut TestAppContext) {
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
    let (window, shell): (AnyWindowHandle, Entity<Shell>) = cx.update(|cx| {
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
                shell.mount(register::<Asker>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    });
    let click = |cx: &mut TestAppContext, id: &'static str| {
        cx.update_window(window, |_, window, cx| {
            window.activate_window();
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
    };

    click(cx, "rail-com.isonomic.asker");
    let tab = shell.read_with(cx, |s, _| s.app(ASKER).cloned()).unwrap();
    wait_until(cx, &tab, |tab| matches!(tab.state(), AppState::Open(_)));
    let asker = tab
        .read_with(cx, |tab, _| match tab.state() {
            AppState::Open(app) => app.view().downcast::<Asker>().ok(),
            _ => None,
        })
        .expect("the tab draws the app");

    // Not from a click: refused, and nothing reaches the engine.
    asker.update(cx, |asker, cx| asker.ask(cx));
    assert_eq!(answers(cx, 1), ["no_user_input"]);
    let mints = |engine: &TestEngine| {
        let calls = engine.calls();
        calls
            .iter()
            .filter(|c| c.method == "approvals.user_action")
            .count()
    };
    assert_eq!(mints(&engine), 0);

    // From the click: the shell's client mints for this app. (The test
    // engine mints without looking the approval up; `decide` checks it.)
    click(cx, "allow");
    assert_eq!(answers(cx, 2), ["no_user_input", "token"]);
    assert_eq!(mints(&engine), 1);

    // After the click is over, the gate is closed again.
    asker.update(cx, |asker, cx| asker.ask(cx));
    assert_eq!(answers(cx, 3), ["no_user_input", "token", "no_user_input"]);
    assert_eq!(mints(&engine), 1);
}
