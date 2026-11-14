//! Isonomic Desktop: the window. Finds or starts the engine, opens the
//! shell, and owns the menu bar icon. Closing the window hides it; Quit
//! stops the engine in order, then the window.

extern crate ui as gpui;

use engine_link::{EngineLink, LinkConfig};
use shell::{lifecycle, menubar, Shell};
use ui::{
    tokens, AppContext as _, Bounds, KeyBinding, Menu, MenuItem, WindowBounds, WindowOptions,
};

ui::actions!(isonomic, [Quit, Hide]);

fn main() {
    let link = LinkConfig::from_environment()
        .and_then(EngineLink::start)
        .unwrap_or_else(|error| {
            eprintln!("[isonomic-desktop] fatal: {error}");
            std::process::exit(1);
        });
    // The link owns the connection: it lives for the whole process.
    let link: &'static EngineLink = Box::leak(Box::new(link));

    let app = ui::application();
    // Dock icon clicked while the window is hidden.
    app.on_reopen(lifecycle::show);
    app.run(move |cx| {
        ui::init(cx);
        let client = link.client().clone();

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-w", Hide, None),
        ]);
        let quit_client = client.clone();
        cx.on_action(move |_: &Quit, cx| lifecycle::quit(quit_client.clone(), cx));
        cx.on_action(|_: &Hide, cx| cx.hide());
        cx.set_menus([Menu::new("Isonomic").items([
            MenuItem::action("Hide Isonomic", Hide),
            MenuItem::separator(),
            MenuItem::action("Quit Isonomic", Quit),
        ])]);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                tokens::window_size(),
                cx,
            ))),
            ..Default::default()
        };
        let live = client.live();
        ui::open_window(options, cx, |window, cx| {
            lifecycle::hide_on_close(window, cx);
            cx.new(|cx| Shell::new(live, cx))
        })
        .expect("failed to open the window");

        menubar::install(client, link.client().live(), cx);
    });
}
