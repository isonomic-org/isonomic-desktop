// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The menu bar icon (architecture spec, section 2): the Isonomic mark, and
//! under it the status line, Open, Start at Login and Quit.
//!
//! The mark is a template image, black on clear, as the system asks of a
//! menu bar icon: the system colours it for a light or dark bar and for
//! the pressed state.
//!
//! GPUI has no API for a status item, so on macOS this owns an
//! `NSStatusItem` through the `tray-icon` crate, in the window process, on
//! the main thread. The status line follows the engine link's live watch; nothing
//! polls. On other platforms `install` does nothing.

use std::sync::Arc;

use isonomic_client::{Client, Live};
use tokio::sync::watch;
use ui::App;

/// The start-at-login switch. The shell knows only on and off; the engine
/// link owns the agent behind it.
pub trait LoginToggle: Send + Sync + 'static {
    fn enabled(&self) -> bool;
    /// Blocks on files and the system's service manager: called off the
    /// foreground thread.
    fn set_enabled(&self, on: bool) -> Result<(), String>;
}

/// Put the icon in the menu bar. Keeps itself alive for the life of the app.
/// Without `login` (a build run from a source tree) the switch isn't shown.
pub fn install(
    client: Client,
    live: watch::Receiver<Live>,
    login: Option<Arc<dyn LoginToggle>>,
    cx: &mut App,
) {
    #[cfg(target_os = "macos")]
    mac::install(client, live, login, cx);
    #[cfg(not(target_os = "macos"))]
    let _ = (client, live, login, cx);
}

#[cfg(target_os = "macos")]
mod mac {
    use std::sync::Arc;

    use isonomic_client::{Client, Live};
    use tokio::sync::{mpsc, watch};
    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
    use ui::{App, AsyncApp};

    use super::LoginToggle;
    use crate::{lifecycle, light};

    const OPEN: &str = "open";
    const LOGIN: &str = "login";
    const QUIT: &str = "quit";

    /// The mark, 36 pixels square: 18 points on a bar at two pixels a point.
    const MARK: &[u8] = include_bytes!("../../../assets/icon/menu-bar-mark.png");

    fn mark() -> Option<Icon> {
        let pixels = image::load_from_memory_with_format(MARK, image::ImageFormat::Png)
            .ok()?
            .into_rgba8();
        let (width, height) = pixels.dimensions();
        Icon::from_rgba(pixels.into_raw(), width, height).ok()
    }

    fn apply(tray: &TrayIcon, status: &MenuItem, live: &Live) {
        let line = light::status_line(live);
        let _ = tray.set_tooltip(Some(&line));
        status.set_text(&line);
    }

    pub fn install(
        client: Client,
        mut live: watch::Receiver<Live>,
        login: Option<Arc<dyn LoginToggle>>,
        cx: &mut App,
    ) {
        let current = live.borrow_and_update().clone();
        let status = MenuItem::new(light::status_line(&current), false, None);
        let menu = Menu::new();
        let open = MenuItem::with_id(OPEN, "Open Isonomic", true, None);
        let at_login = login.as_ref().map(|login| {
            CheckMenuItem::with_id(LOGIN, "Start at Login", true, login.enabled(), None)
        });
        let quit = MenuItem::with_id(QUIT, "Quit Isonomic", true, None);
        let separator = PredefinedMenuItem::separator();
        let built = match &at_login {
            Some(at_login) => menu.append_items(&[&status, &separator, &open, at_login, &quit]),
            None => menu.append_items(&[&status, &separator, &open, &quit]),
        };
        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(light::status_line(&current));
        if let Some(icon) = mark() {
            builder = builder.with_icon_templated(icon);
        }
        let tray = match built
            .map_err(|e| e.to_string())
            .and_then(|_| builder.build().map_err(|e| e.to_string()))
        {
            Ok(tray) => tray,
            Err(error) => {
                eprintln!("[isonomic-desktop] menu bar icon unavailable: {error}");
                return;
            }
        };

        // Menu clicks arrive on the main thread; hand them to the GPUI task.
        let (clicks, mut clicked) = mpsc::unbounded_channel();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = clicks.send(event.id.0);
        }));

        cx.spawn(async move |cx: &mut AsyncApp| {
            // The tray and its items live here, on the main thread, for good.
            let (tray, status, at_login) = (tray, status, at_login);
            loop {
                tokio::select! {
                    changed = live.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        let latest = live.borrow_and_update().clone();
                        apply(&tray, &status, &latest);
                    }
                    Some(id) = clicked.recv() => {
                        if id == LOGIN {
                            if let (Some(login), Some(item)) = (&login, &at_login) {
                                // Flip what is installed, not what the check
                                // shows, then show what is installed.
                                let toggle = Arc::clone(login);
                                let (outcome, now) = cx
                                    .background_executor()
                                    .spawn(async move {
                                        let outcome = toggle.set_enabled(!toggle.enabled());
                                        (outcome, toggle.enabled())
                                    })
                                    .await;
                                if let Err(error) = outcome {
                                    eprintln!("[isonomic-desktop] start at login: {error}");
                                }
                                item.set_checked(now);
                            }
                            continue;
                        }
                        let client = client.clone();
                        cx.update(|cx| match id.as_str() {
                            OPEN => lifecycle::show(cx),
                            QUIT => lifecycle::quit(client, cx),
                            _ => {}
                        });
                    }
                }
            }
            // Keep the icon even if the link's watch ends.
            std::future::pending::<()>().await;
            drop((tray, status, at_login));
        })
        .detach();
    }
}
