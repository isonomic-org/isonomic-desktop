//! Isonomic Desktop: the window. Finds or starts the engine, then opens the
//! shell. Everything it shows about the engine arrives through the link's
//! live watch.

use engine_link::{EngineLink, LinkConfig};
use shell::Shell;
use ui::{tokens, AppContext as _, Bounds, WindowBounds, WindowOptions};

fn main() {
    let link = LinkConfig::from_environment()
        .and_then(EngineLink::start)
        .unwrap_or_else(|error| {
            eprintln!("[isonomic-desktop] fatal: {error}");
            std::process::exit(1);
        });
    // The link owns the connection: it lives for the whole process.
    let link: &'static EngineLink = Box::leak(Box::new(link));

    ui::application().run(move |cx| {
        ui::init(cx);
        let live = link.client().live();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                tokens::window_size(),
                cx,
            ))),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| cx.new(|cx| Shell::new(live, cx)))
            .expect("failed to open the window");
    });
}
