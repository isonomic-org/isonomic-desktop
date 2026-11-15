//! Isonomic Desktop: the window. Finds or starts the engine, opens the
//! shell, and owns the menu bar icon. Closing the window hides it; Quit
//! stops the engine in order, then the window.

extern crate ui as gpui;

use std::sync::Arc;

use engine_link::agent::LoginAgent;
use engine_link::{EngineLink, LinkConfig};
use shell::menubar::LoginToggle;
use shell::{lifecycle, menubar, Shell};
use ui::{
    tokens, AppContext as _, Bounds, KeyBinding, Menu, MenuItem, WindowBounds, WindowOptions,
};

ui::actions!(isonomic, [Quit, Hide]);

/// The menu's switch, backed by the login agent.
struct AtLogin(Arc<LoginAgent>);

impl LoginToggle for AtLogin {
    fn enabled(&self) -> bool {
        self.0.enabled()
    }

    fn set_enabled(&self, on: bool) -> Result<(), String> {
        self.0
            .set_enabled(on)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn main() {
    let config = LinkConfig::from_environment().unwrap_or_else(|error| {
        eprintln!("[isonomic-desktop] fatal: {error}");
        std::process::exit(1);
    });
    // Before the link and before the event loop: a missing engine is then
    // started through the agent, and nothing here blocks a frame.
    let agent = config.login_agent().map(Arc::new);
    if let Some(agent) = &agent {
        match agent.sync() {
            Ok(change) => eprintln!("[isonomic-desktop] start at login: {change:?}"),
            Err(error) => eprintln!("[isonomic-desktop] start at login not set up: {error}"),
        }
    }
    let link = EngineLink::start(config, agent.clone()).unwrap_or_else(|error| {
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
        let logs = client.clone();
        ui::open_window(options, cx, |window, cx| {
            lifecycle::hide_on_close(window, cx);
            cx.new(|cx| Shell::new(live, Arc::new(logs), cx))
        })
        .expect("failed to open the window");

        let login = agent.map(|agent| Arc::new(AtLogin(agent)) as Arc<dyn LoginToggle>);
        menubar::install(client, link.client().live(), login, cx);
    });
}
