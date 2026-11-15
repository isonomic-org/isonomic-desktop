//! The window's shell (architecture spec, sections 2 and 6): a rail of tabs
//! down the left and a host that shows the selected one.
//!
//! Each tab is an entity that is built once and stays alive. The shell knows
//! nothing about the engine beyond the live status watch and the log source
//! it hands to the Home tab; it never touches a process, a socket or a path.
//! First-party apps are mounted as further tabs through [`apps`].

pub mod apps;
pub mod downloads;
pub mod first_run;
pub mod home;
pub mod lifecycle;
pub mod light;
pub mod logs;
pub mod menubar;

use std::sync::Arc;

use isonomic_app::{Attention, Registration};
use isonomic_client::Live;
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, tokens, AnyView, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _, Window,
};

use apps::{AppHost, AppTab};
use first_run::{Finished, FirstRun, Setup};
use home::HomeTab;
use logs::LogSource;

/// One entry in the rail and the view it opens.
struct Tab {
    /// Stable identifier, used for the rail item's element id: `home`, or
    /// an app's id.
    id: SharedString,
    title: SharedString,
    view: AnyView,
    /// Set for an app's tab.
    app: Option<Entity<AppTab>>,
}

pub struct Shell {
    home: Entity<HomeTab>,
    tabs: Vec<Tab>,
    selected: usize,
    /// Shown in place of the tabs until it finishes.
    first_run: Option<Entity<FirstRun>>,
    /// How many apps stay loaded; `None` until the profile is known, and
    /// on machines with no cap.
    cap: Option<usize>,
    /// App tabs by index, least recently opened first.
    recent: Vec<usize>,
}

impl Shell {
    /// Build the shell with its system tabs. `live` is the engine link's
    /// watch on connection and service status; `logs` answers `system.logs`.
    pub fn new(
        live: watch::Receiver<Live>,
        logs: Arc<dyn LogSource>,
        cx: &mut Context<Self>,
    ) -> Self {
        let home = cx.new(|cx| HomeTab::new(live, logs, cx));
        Self {
            home: home.clone(),
            tabs: vec![Tab {
                id: "home".into(),
                title: "Home".into(),
                view: home.into(),
                app: None,
            }],
            selected: 0,
            first_run: None,
            cap: None,
            recent: Vec::new(),
        }
    }

    /// Cap how many apps stay loaded (see [`apps::loaded_cap`]).
    pub fn set_loaded_cap(&mut self, cap: Option<usize>, cx: &mut Context<Self>) {
        self.cap = cap;
        self.enforce_cap(cx);
    }

    /// Set the cap from the machine's hardware profile once the engine
    /// answers. Until then, no app is unloaded.
    pub fn follow_profile(
        &mut self,
        profile: futures::future::BoxFuture<'static, Result<String, String>>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |shell, cx| {
            if let Ok(profile) = profile.await {
                let _ = shell.update(cx, |shell, cx| {
                    shell.set_loaded_cap(apps::loaded_cap(&profile), cx)
                });
            }
        })
        .detach();
    }

    /// Unload the least recently opened apps past the cap, never the one
    /// on screen.
    fn enforce_cap(&mut self, cx: &mut Context<Self>) {
        let Some(cap) = self.cap else {
            return;
        };
        let loaded: Vec<usize> = self
            .recent
            .iter()
            .copied()
            .filter(|&index| self.loaded(index, cx))
            .collect();
        let excess = loaded.len().saturating_sub(cap);
        for &index in loaded.iter().filter(|&&i| i != self.selected).take(excess) {
            if let Some(app) = &self.tabs[index].app {
                app.update(cx, |app, cx| app.unload(cx));
            }
        }
    }

    fn loaded(&self, index: usize, cx: &Context<Self>) -> bool {
        self.tabs[index]
            .app
            .as_ref()
            .is_some_and(|app| app.read(cx).is_loaded())
    }

    /// Show the first-run flow until it finishes. `created`: the Capsule
    /// was made at this launch.
    pub fn start_first_run(
        &mut self,
        setup: Arc<dyn Setup>,
        created: bool,
        cx: &mut Context<Self>,
    ) {
        let first_run = cx.new(|cx| FirstRun::new(setup, created, cx));
        cx.subscribe(&first_run, |shell, _, _: &Finished, cx| {
            shell.first_run = None;
            cx.notify();
        })
        .detach();
        self.first_run = Some(first_run);
        cx.notify();
    }

    /// The first-run flow, while it is showing.
    pub fn first_run(&self) -> Option<&Entity<FirstRun>> {
        self.first_run.as_ref()
    }

    /// The Home tab.
    pub fn home(&self) -> &Entity<HomeTab> {
        &self.home
    }

    /// Identifier of the tab on screen.
    pub fn selected_tab(&self) -> &str {
        &self.tabs[self.selected].id
    }

    /// Add an app crate's tab to the rail. The app is built when the tab is
    /// first opened.
    pub fn mount(&mut self, registration: Registration, host: AppHost, cx: &mut Context<Self>) {
        let manifest = registration.manifest();
        let (id, title) = (manifest.id.clone(), manifest.name.clone());
        let app = apps::tab(registration, host, cx);
        // The rail redraws when the app asks for attention.
        cx.observe(&app, |_, _, cx| cx.notify()).detach();
        self.tabs.push(Tab {
            id: id.into(),
            title: title.into(),
            view: app.clone().into(),
            app: Some(app),
        });
        cx.notify();
    }

    /// Open a deep link: select the app it names, building it if needed,
    /// and hand it the link. `false` if no mounted app has that name.
    pub fn open_link(&mut self, url: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(target) = apps::link_target(url) else {
            return false;
        };
        let found = self
            .tabs
            .iter()
            .position(|tab| tab.app.is_some() && apps::link_segment(&tab.id) == target);
        let Some(index) = found else {
            return false;
        };
        if let Some(app) = self.tabs[index].app.clone() {
            app.update(cx, |app, cx| app.open_link(url, cx));
        }
        if index == self.selected {
            // Already on screen; make sure it is built.
            if let Some(app) = &self.tabs[index].app {
                app.update(cx, |app, cx| app.activate(window, cx));
            }
        } else {
            self.select(index, window, cx);
        }
        true
    }

    /// Search every loaded app, in rail order. Each hit names its app.
    /// Unloaded apps are not asked.
    pub fn search(
        &self,
        q: &str,
        cx: &mut Context<Self>,
    ) -> Vec<(SharedString, isonomic_app::SearchHit)> {
        let mut hits = Vec::new();
        for tab in &self.tabs {
            if let Some(app) = &tab.app {
                let found = app.update(cx, |app, cx| app.search(q, cx));
                hits.extend(found.into_iter().map(|hit| (tab.id.clone(), hit)));
            }
        }
        hits
    }

    /// The tab of the app with `id`, if it is mounted.
    pub fn app(&self, id: &str) -> Option<&Entity<AppTab>> {
        self.tabs.iter().find(|tab| tab.id == id)?.app.as_ref()
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() || index == self.selected {
            return;
        }
        self.selected = index;
        if let Some(app) = &self.tabs[index].app {
            app.update(cx, |app, cx| app.activate(window, cx));
            self.recent.retain(|&i| i != index);
            self.recent.push(index);
            self.enforce_cap(cx);
        }
        cx.notify();
    }
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(first_run) = &self.first_run {
            return div().id("shell").size_full().child(first_run.clone());
        }
        let theme = cx.theme();
        let (accent, accent_text) = (theme.sidebar_accent, theme.sidebar_accent_foreground);
        let (badge, badge_text) = (theme.primary, theme.primary_foreground);
        let rail_items: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let selected = index == self.selected;
                // A monogram stands in for the icon until the icon set lands.
                let monogram: SharedString = tab.title.chars().take(1).collect::<String>().into();
                let attention = tab
                    .app
                    .as_ref()
                    .map_or(Attention::None, |app| app.read(cx).attention());
                let mut item = div()
                    .id(SharedString::from(format!("rail-{}", tab.id)))
                    .relative()
                    .test_support()
                    .size_9()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .child(monogram)
                    .on_click(
                        cx.listener(move |shell, _, window, cx| shell.select(index, window, cx)),
                    );
                if selected {
                    item = item.bg(accent).text_color(accent_text);
                }
                let marker = match attention {
                    Attention::None => None,
                    Attention::Dot => Some(div().size(tokens::status_dot())),
                    Attention::Count(n) => Some(div().px_1().text_xs().child(n.to_string())),
                };
                if let Some(marker) = marker {
                    item = item.child(
                        marker
                            .id(SharedString::from(format!("attention-{}", tab.id)))
                            .test_support()
                            .absolute()
                            .top_0()
                            .right_0()
                            .rounded_full()
                            .bg(badge)
                            .text_color(badge_text),
                    );
                }
                item
            })
            .collect();

        h_flex()
            .id("shell")
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                v_flex()
                    .id("rail")
                    .test_support()
                    .w(tokens::rail_width())
                    .h_full()
                    .flex_shrink_0()
                    .items_center()
                    .gap_1()
                    .py_2()
                    .bg(theme.sidebar)
                    .text_color(theme.sidebar_foreground)
                    .border_r_1()
                    .border_color(theme.sidebar_border)
                    .children(rail_items),
            )
            .child(
                div()
                    .id("tab-host")
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(self.tabs[self.selected].view.clone()),
            )
    }
}
