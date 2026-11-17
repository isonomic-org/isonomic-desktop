// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The check build's icon check, rendered for real: GPUI's headless
//! renderer (Metal on macOS; on Linux a software Vulkan driver such as
//! Mesa lavapipe) draws the real shell through first run, and the pixels
//! of the rail's Home icon are counted the way the self-check counts them.
//! Built only with `--features check-build`, and skipped where no headless
//! renderer starts.

use std::borrow::Cow;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_client::Live;
use isonomic_proto::models::{DownloadProgress, Recommendation};
use isonomic_proto::HardwareProfile;
use shell::first_run::{Setup, MEMORY_FLOOR_BYTES};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, AssetSource, Entity, HeadlessAppContext, SharedString};

use super::screens::ink_in;

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// First run with everything in place: enough memory, an ID, the model.
struct Ready;

impl Setup for Ready {
    fn profile(&self) -> BoxFuture<'static, Result<HardwareProfile, String>> {
        Box::pin(async {
            Ok(HardwareProfile {
                profile: "standard".into(),
                total_memory_bytes: MEMORY_FLOOR_BYTES * 2,
                available_memory_bytes: MEMORY_FLOOR_BYTES,
            })
        })
    }

    fn recommended(&self) -> BoxFuture<'static, Result<Recommendation, String>> {
        Box::pin(async {
            Ok(Recommendation {
                model: "chat-small".into(),
                profile: "standard".into(),
                size_bytes: None,
            })
        })
    }

    fn installed(&self, _: &str) -> BoxFuture<'static, Result<bool, String>> {
        Box::pin(async { Ok(true) })
    }

    fn download(&self, _: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        Box::pin(futures::stream::empty())
    }

    fn cancel_download(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn identity(&self) -> BoxFuture<'static, Result<String, String>> {
        Box::pin(async { Ok("did:iso:0123456789abcdef".into()) })
    }

    fn finish(&self) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// An asset source with no icons, as the window had before #61.
struct NoIcons;

impl AssetSource for NoIcons {
    fn load(&self, path: &str) -> ui::Result<Option<Cow<'static, [u8]>>> {
        if path.starts_with("icons/") {
            Ok(None)
        } else {
            ui::assets::AllAssets.load(path)
        }
    }

    fn list(&self, path: &str) -> ui::Result<Vec<SharedString>> {
        ui::assets::AllAssets.list(path)
    }
}

/// The shell in a headless window with a real renderer, first run up.
fn open(
    assets: Arc<dyn AssetSource>,
) -> Option<(HeadlessAppContext, AnyWindowHandle, Entity<Shell>)> {
    // No GPU and no software driver: no renderer starts, and the test is
    // skipped rather than failed.
    let mut cx = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let platform = ui::platform::current_platform(true);
        HeadlessAppContext::with_platform(
            platform.text_system(),
            assets,
            ui::platform::current_headless_renderer,
        )
    }))
    .ok()?;
    cx.update(ui::init);
    let (_, live) = watch::channel(Live::default());
    let (window, shell) = cx
        .update(|cx| {
            ui::open_window(
                ui::WindowOptions {
                    window_bounds: Some(ui::WindowBounds::Windowed(ui::Bounds {
                        origin: Default::default(),
                        size: ui::tokens::window_size(),
                    })),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                cx,
                |_, cx| {
                    cx.new(|cx| {
                        let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                        shell.start_first_run(Arc::new(Ready), true, cx);
                        shell
                    })
                },
            )
        })
        .ok()?;
    // No renderer here (no GPU, no software driver): nothing to measure.
    cx.capture_screenshot(window).ok()?;
    Some((cx, window, shell))
}

/// First run's checks settle; Continue, as the self-check presses it.
fn finish_first_run(cx: &mut HeadlessAppContext, window: AnyWindowHandle, shell: &Entity<Shell>) {
    cx.run_until_parked();
    let first_run = cx
        .update(|cx| shell.read(cx).first_run().cloned())
        .expect("first run is up");
    cx.update(|cx| first_run.update(cx, |run, cx| run.continue_if_ready(cx)));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    assert!(
        cx.update(|cx| shell.read(cx).first_run().is_none()),
        "first run finished"
    );
}

#[test]
fn the_rail_icon_draws_after_first_run_and_not_without_icons() {
    let Some((mut cx, window, shell)) = open(Arc::new(ui::assets::AllAssets)) else {
        eprintln!("skipped: no headless renderer here");
        return;
    };
    // While first run is up there is no rail: the check can't run yet,
    // which is what failed on the Mac when it ran at connect.
    let during = cx.update(|cx| ink_in(window, "rail-home", cx));
    assert!(during.is_err(), "{during:?}");
    finish_first_run(&mut cx, window, &shell);
    let ink = cx.update(|cx| ink_in(window, "rail-home", cx)).unwrap();
    eprintln!("the rail's Home icon drew {ink} pixels");
    assert!(ink > 0, "the Home icon drew nothing");

    let (mut bare, window, shell) = open(Arc::new(NoIcons)).expect("a renderer, as above");
    finish_first_run(&mut bare, window, &shell);
    let none = bare.update(|cx| ink_in(window, "rail-home", cx)).unwrap();
    assert_eq!(
        none, 0,
        "with no icons, nothing should draw in the rail's Home button"
    );
}
