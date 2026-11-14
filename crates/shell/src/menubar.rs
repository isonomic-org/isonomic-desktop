//! The menu bar icon (architecture spec, section 2): a status light, Open
//! and Quit.
//!
//! GPUI has no API for a status item, so on macOS this owns an
//! `NSStatusItem` through the `tray-icon` crate, in the window process, on
//! the main thread. The light follows the engine link's live watch; nothing
//! polls. On other platforms `install` does nothing.

use isonomic_client::{Client, Live};
use tokio::sync::watch;
use ui::App;

/// Put the icon in the menu bar. Keeps itself alive for the life of the app.
pub fn install(client: Client, live: watch::Receiver<Live>, cx: &mut App) {
    #[cfg(target_os = "macos")]
    mac::install(client, live, cx);
    #[cfg(not(target_os = "macos"))]
    let _ = (client, live, cx);
}

#[cfg(target_os = "macos")]
mod mac {
    use isonomic_client::{Client, Live};
    use tokio::sync::{mpsc, watch};
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
    use ui::tokens::light::ICON_SIZE;
    use ui::{App, AsyncApp};

    use crate::{lifecycle, light};

    const OPEN: &str = "open";
    const QUIT: &str = "quit";

    fn icon_for(live: &Live) -> Option<Icon> {
        Icon::from_rgba(light::icon(light::color(live)), ICON_SIZE, ICON_SIZE).ok()
    }

    fn apply(tray: &TrayIcon, status: &MenuItem, live: &Live) {
        let _ = tray.set_icon(icon_for(live));
        let line = light::status_line(live);
        let _ = tray.set_tooltip(Some(&line));
        status.set_text(&line);
    }

    pub fn install(client: Client, mut live: watch::Receiver<Live>, cx: &mut App) {
        let current = live.borrow_and_update().clone();
        let status = MenuItem::new(light::status_line(&current), false, None);
        let menu = Menu::new();
        let open = MenuItem::with_id(OPEN, "Open Isonomic", true, None);
        let quit = MenuItem::with_id(QUIT, "Quit Isonomic", true, None);
        let built = menu.append_items(&[&status, &PredefinedMenuItem::separator(), &open, &quit]);
        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(light::status_line(&current));
        if let Some(icon) = icon_for(&current) {
            builder = builder.with_icon(icon);
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
            let (tray, status) = (tray, status);
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
            drop((tray, status));
        })
        .detach();
    }
}
