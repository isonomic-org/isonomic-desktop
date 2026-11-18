// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The launch screen in a headless window, in the real shell: it stands in
//! for everything until its intro ends, then the window shows its tabs.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::Live;
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
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

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Shell>) {
    let (_publish, live) = watch::channel(Live::default());
    cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.), px(800.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.start_launch(cx);
                shell
            })
        })
        .expect("open test window")
    })
}

fn shown(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window
            .try_find(id)
            .map(|element| element.visible())
            .unwrap_or(false)
    })
    .unwrap()
}

#[ui::test]
fn the_window_opens_on_the_launch_screen_alone(cx: &mut TestAppContext) {
    let (window, shell) = open(cx);
    assert!(shown(cx, window, "launch"));
    assert!(!shown(cx, window, "rail"), "nothing else is on the page");
    assert!(shell.read_with(cx, |shell, _| shell.launch().is_some()));
}

#[ui::test]
fn when_the_intro_ends_the_window_shows_its_tabs(cx: &mut TestAppContext) {
    let (window, shell) = open(cx);
    assert!(shown(cx, window, "launch"));
    let launch = shell.read_with(cx, |shell, _| shell.launch().cloned().unwrap());
    launch.update(cx, |launch, cx| {
        launch.finish(cx);
        // Ending it twice changes nothing.
        launch.finish(cx);
    });
    cx.run_until_parked();
    assert!(!shown(cx, window, "launch"));
    assert!(shown(cx, window, "rail"));
    assert!(shell.read_with(cx, |shell, _| shell.launch().is_none()));
}
