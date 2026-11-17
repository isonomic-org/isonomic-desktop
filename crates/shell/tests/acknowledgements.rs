// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Acknowledgements dialog shows the notices it is given, or says the
//! build has none.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::Live;
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::component::WindowExt as _;
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, TestAppContext, WindowOptions};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

fn window(cx: &mut TestAppContext) -> AnyWindowHandle {
    let (_, live) = watch::channel(Live::default());
    cx.update(|cx| {
        ui::init(cx);
        ui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| Shell::new(live, Arc::new(NoLogs), cx))
        })
        .expect("open test window")
        .0
    })
}

fn shows(cx: &mut TestAppContext, window: AnyWindowHandle, notices: Option<&str>) -> Vec<bool> {
    let notices = notices.map(str::to_string);
    cx.update_window(window, |_, window, cx| {
        shell::acknowledgements::open(notices, window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        let seen = ["acknowledgements-text", "acknowledgements-missing"]
            .map(|id| window.try_find(id).is_some_and(|e| e.visible()));
        window.click("acknowledgements-done", cx);
        seen.to_vec()
    })
    .unwrap()
}

#[ui::test]
fn the_notices_are_shown_or_said_to_be_missing(cx: &mut TestAppContext) {
    let window = window(cx);
    assert_eq!(
        shows(cx, window, Some("database 17\nlicence text")),
        [true, false]
    );
    cx.run_until_parked();
    assert!(!cx
        .update_window(window, |_, window, cx| window.has_active_dialog(cx))
        .unwrap());
    assert_eq!(shows(cx, window, None), [false, true]);
}
