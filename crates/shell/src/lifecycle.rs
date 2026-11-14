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
use ui::{App, AsyncApp, Window};

/// How long Quit waits for the engine to finish stopping before the window
/// exits anyway. The engine keeps stopping in order on its own after that.
pub const QUIT_WAIT: Duration = Duration::from_secs(20);

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
