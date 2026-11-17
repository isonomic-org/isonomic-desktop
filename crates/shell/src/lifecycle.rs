// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What closing, reopening and quitting do (architecture spec, section 2):
//!
//! - Closing the window hides it. The process stays resident for the menu
//!   bar icon; the engine keeps running.
//! - Reopening (Dock icon, menu bar Open) shows it again.
//! - Quit asks the engine to stop every service in order, waits until it
//!   has, then ends the window process.

use std::time::Duration;

use futures::future::{select, Either};
use isonomic_client::Client;
use ui::{
    size, tokens, App, AsyncApp, Bounds, Pixels, Size, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};

/// How long Quit waits for the engine to finish stopping before the window
/// exits anyway. The engine keeps stopping in order on its own after that.
pub const QUIT_WAIT: Duration = Duration::from_secs(20);

/// The window's title, as macOS shows it in the title bar, the Window menu
/// and the app switcher.
pub const TITLE: &str = "Isonomic";

/// The size to open at: the one last kept, else the design's
/// ([`tokens::window_size`], 1440 × 900), never larger than `screen`.
pub fn opening_size(
    remembered: Option<Size<Pixels>>,
    screen: Option<Size<Pixels>>,
) -> Size<Pixels> {
    let wanted = remembered.unwrap_or_else(tokens::window_size);
    match screen {
        Some(fit) => size(wanted.width.min(fit.width), wanted.height.min(fit.height)),
        None => wanted,
    }
}

/// How the main window opens: centred, titled, at [`opening_size`] for the
/// screen it opens on.
pub fn window_options(cx: &App, remembered: Option<Size<Pixels>>) -> WindowOptions {
    let screen = cx.primary_display().map(|d| d.visible_bounds().size);
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            opening_size(remembered, screen),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some(TITLE.into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Make the close button hide the app instead of closing the window.
pub fn hide_on_close(window: &mut Window, cx: &mut App) {
    window.on_window_should_close(cx, |_, cx| {
        cx.hide();
        false
    });
}

/// Bring the app and its windows to the front.
pub fn show(cx: &mut App) {
    cx.activate(true);
    for handle in cx.windows() {
        let _ = handle.update(cx, |_, window, _| window.activate_window());
    }
}

/// Stop the engine in order, then quit. Returns at once; the work runs on
/// GPUI's foreground executor and never blocks it.
pub fn quit(client: Client, cx: &mut App) {
    cx.spawn(async move |cx: &mut AsyncApp| {
        stop_engine(&client, cx).await;
        cx.update(|cx| cx.quit());
    })
    .detach();
}

/// Ask the engine to shut down and wait until it has dropped the
/// connection, or `QUIT_WAIT` passes. Nothing to do if it is not connected.
async fn stop_engine(client: &Client, cx: &mut AsyncApp) {
    let mut live = client.live();
    if !live.borrow().connected || client.shutdown().await.is_err() {
        return;
    }
    let timer = cx.background_executor().timer(QUIT_WAIT);
    let gave_up = {
        let stopped = Box::pin(live.wait_for(|view| !view.connected));
        matches!(select(stopped, timer).await, Either::Right(_))
    };
    if gave_up {
        eprintln!("[isonomic-desktop] engine still stopping after {QUIT_WAIT:?}; quitting anyway");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::px;

    #[test]
    fn the_window_opens_at_the_design_size_or_the_last_one_and_always_fits() {
        let mac13 = Some(size(px(1440.), px(875.)));
        assert_eq!(opening_size(None, None), size(px(1440.), px(900.)));
        assert_eq!(opening_size(None, mac13), size(px(1440.), px(875.)));
        let kept = Some(size(px(1200.), px(800.)));
        assert_eq!(opening_size(kept, mac13), size(px(1200.), px(800.)));
        let big = Some(size(px(2400.), px(1400.)));
        assert_eq!(opening_size(big, mac13), size(px(1440.), px(875.)));
    }
}
