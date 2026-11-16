//! The window's screens with Isonomic UI's QA-passed components: reachable
//! by keyboard, drawn in both themes, and the checks QA could not cover
//! (scrolling a focused control into view).

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_app::{register, App, AppContext, Capabilities, Manifest, MemorySettings};
use isonomic_client::Live;
use isonomic_testkit::TestEngine;
use shell::apps::AppHost;
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::component::theme::{Theme, ThemeMode};
use ui::test::TestWindowExt as _;
use ui::{
    div, px, size, AnyView, AnyWindowHandle, AppContext as _, Bounds, Context, Entity, IntoElement,
    ParentElement as _, Point, Render, TestAppContext, Window, WindowBounds, WindowOptions,
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

macro_rules! probe {
    ($name:ident, $id:literal) => {
        struct $name;
        impl App for $name {
            fn manifest() -> Manifest {
                Manifest {
                    id: $id.into(),
                    name: stringify!($name).into(),
                    version: "0.1.0".into(),
                    icon: "cpu".into(),
                    capabilities: Capabilities::default(),
                    schemas: vec![],
                }
            }
            fn build(_: AppContext, _: &mut Window, app: &mut ui::App) -> AnyView {
                app.new(|_| $name).into()
            }
        }
        impl Render for $name {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().child(stringify!($name))
            }
        }
    };
}

probe!(Alpha, "com.isonomic.test-alpha");
probe!(Beta, "com.isonomic.test-beta");
probe!(Gamma, "com.isonomic.test-gamma");
probe!(Delta, "com.isonomic.test-delta");
probe!(Epsilon, "com.isonomic.test-epsilon");
probe!(Zeta, "com.isonomic.test-zeta");

const LAST: &str = "rail-com.isonomic.test-zeta";

struct Harness {
    window: AnyWindowHandle,
    shell: Entity<Shell>,
    _engine: TestEngine,
    _runtime: tokio::runtime::Runtime,
}

/// The shell with `apps` copies of the probe app, in a window `height` tall.
fn open(cx: &mut TestAppContext, apps: usize, height: f32) -> Harness {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let host = AppHost::new(
        engine.client_on(runtime.handle()),
        Arc::new(MemorySettings::default()),
    );
    let (_, live) = watch::channel(Live::default());
    // Opening an app grants its token on the client's own thread.
    cx.executor().allow_parking();
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(900.), px(height)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                let all = [
                    register::<Alpha>(),
                    register::<Beta>(),
                    register::<Gamma>(),
                    register::<Delta>(),
                    register::<Epsilon>(),
                    register::<Zeta>(),
                ];
                for app in all.into_iter().take(apps) {
                    shell.mount(app, host.clone(), cx);
                }
                shell
            })
        })
        .expect("open test window")
    });
    Harness {
        window,
        shell,
        _engine: engine,
        _runtime: runtime,
    }
}

/// Real key presses, down then up, as a person types them, starting from
/// the focus the window gives the rail when it opens.
fn press(cx: &mut TestAppContext, h: &Harness, keys: &'static str) {
    let mut visual = ui::VisualTestContext::from_window(h.window, cx);
    let shell = h.shell.clone();
    visual.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
        if window.focused(cx).is_none() {
            shell.update(cx, |s, cx| s.focus_rail(window, cx));
        }
    });
    for key in keys.split_whitespace() {
        let keystroke = ui::Keystroke::parse(key).unwrap();
        visual.simulate_event(ui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        visual.simulate_event(ui::KeyUpEvent { keystroke });
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }
    visual.run_until_parked();
}

#[ui::test]
fn the_rail_is_reachable_by_keyboard(cx: &mut TestAppContext) {
    let h = open(cx, 1, 600.);
    // The window starts focus on Home's button: Tab to the app, Enter opens it.
    press(cx, &h, "tab enter");
    let selected = h.shell.read_with(cx, |s, _| s.selected_tab().to_string());
    assert_eq!(selected, "com.isonomic.test-alpha");
}

#[ui::test]
fn every_screen_draws_in_both_themes(cx: &mut TestAppContext) {
    let h = open(cx, 1, 600.);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        cx.update(|cx| Theme::change(mode, None, cx));
        cx.update_window(h.window, |_, window, cx| window.render_frame(cx))
            .unwrap();
        let shown = cx
            .update_window(h.window, |_, window, cx| {
                window.render_frame(cx);
                window.try_find("rail-home").map(|e| e.visible())
            })
            .unwrap();
        assert_eq!(shown, Some(true), "{mode:?}");
    }
}

/// QA could not check whether focusing a control that is scrolled out of
/// view brings it into view. The rail scrolls when it holds more apps than
/// the window is tall: Tab to the last app and see whether its button is
/// inside the rail's visible bounds.
#[ui::test]
fn a_focused_rail_button_scrolls_into_view(cx: &mut TestAppContext) {
    let h = open(cx, 6, 160.);
    let bounds = |cx: &mut TestAppContext| {
        cx.update_window(h.window, |_, window, cx| {
            window.render_frame(cx);
            let rail = window.try_find("rail").map(|e| e.bounds()).unwrap();
            let last = window.try_find(LAST).map(|e| e.bounds()).unwrap();
            (rail, last)
        })
        .unwrap()
    };
    let (rail, last) = bounds(cx);
    assert!(
        last.bottom() > rail.bottom(),
        "the test needs the last app below the fold: rail {rail:?}, last {last:?}"
    );
    // From Home's button, six Tabs reach the last app.
    press(cx, &h, "tab tab tab tab tab tab enter");
    let selected = h.shell.read_with(cx, |s, _| s.selected_tab().to_string());
    assert_eq!(
        selected, "com.isonomic.test-zeta",
        "Tab reached the last app"
    );
    let (rail, last) = bounds(cx);
    assert!(
        last.bottom() <= rail.bottom() && last.top() >= rail.top(),
        "the focused button stays out of view: rail {rail:?}, last {last:?}"
    );
}
