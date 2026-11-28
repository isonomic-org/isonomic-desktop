// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window's shell (architecture spec, sections 2 and 6): a rail of tabs
//! down the left and a host that shows the selected one.
//!
//! Each tab is an entity that is built once and stays alive. The shell knows
//! nothing about the engine beyond the live status watch and the log source
//! it hands to the Home tab; it never touches a process, a socket or a path.
//! First-party apps are mounted as further tabs through [`apps`].

pub mod acknowledgements;
pub mod apps;
pub mod connectors;
pub mod downloads;
pub mod first_run;
pub mod home;
pub mod install;
pub mod lifecycle;
pub mod light;
pub mod lists;
pub mod logos;
pub mod logs;
pub mod menubar;
pub mod onboarding;
pub mod problems;
pub mod user_input;
pub mod workspace;

use std::sync::Arc;

use isonomic_app::{Attention, Registration};
use isonomic_client::Live;
use tokio::sync::watch;
use ui::base::{h_flex, v_flex};
use ui::component::badge::Badge;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::theme::ActiveTheme as _;
use ui::component::{Icon, Selectable as _, Sizable as _};
use ui::{
    div, tokens, AnyView, AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

use apps::{AppHost, AppTab};
use first_run::{Finished, FirstRun, Setup};
use home::HomeTab;
use install::{InstallOutcome, ModelInstall, ModelSource};
use logs::LogSource;
use problems::ProblemReports;

/// One entry in the rail and the view it opens.
struct Tab {
    /// Stable identifier, used for the rail item's element id: `home`, or
    /// an app's id.
    id: SharedString,
    title: SharedString,
    /// The icon's name in Isonomic UI's catalog, as a manifest names it.
    icon: SharedString,
    /// The rail button's focus, so keyboard focus can be followed.
    focus: FocusHandle,
    view: AnyView,
    /// Set for an app's tab.
    app: Option<Entity<AppTab>>,
}

/// Told the id of the tab on screen.
type TabListener = Box<dyn Fn(&str)>;

pub struct Shell {
    home: Entity<HomeTab>,
    tabs: Vec<Tab>,
    selected: usize,
    /// Told the id of the tab on screen whenever it changes.
    on_tab: Option<TabListener>,
    /// The rail scrolls when it holds more apps than the window is tall.
    rail_scroll: ScrollHandle,
    /// The rail button last scrolled into view because it had focus.
    rail_followed: Option<usize>,
    /// Shown in place of the tabs until it finishes.
    first_run: Option<Entity<FirstRun>>,
    /// A model an app asked for, shown in place of the tab on screen.
    install: Option<Entity<ModelInstall>>,
    /// How many apps stay loaded; `None` until the profile is known, and
    /// on machines with no cap.
    cap: Option<usize>,
    /// App tabs by index, least recently opened first.
    recent: Vec<usize>,
    /// Problem reports, once the window has handed them over.
    reports: Option<Entity<ProblemReports>>,
    /// Look at the kept reports at the next draw.
    reports_due: bool,
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
                icon: "home".into(),
                focus: cx.focus_handle(),
                view: home.into(),
                app: None,
            }],
            selected: 0,
            on_tab: None,
            rail_scroll: ScrollHandle::new(),
            rail_followed: None,
            first_run: None,
            install: None,
            cap: None,
            recent: Vec::new(),
            reports: None,
            reports_due: false,
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

    /// An app asked for `model`: show its download in place of the tab on
    /// screen. Resolves with how it ended. A second request while one is
    /// showing replaces it, and the first resolves `Declined`.
    pub fn install_model(
        &mut self,
        model: String,
        app: String,
        source: Arc<dyn ModelSource>,
        cx: &mut Context<Self>,
    ) -> futures::channel::oneshot::Receiver<InstallOutcome> {
        let (answer, answered) = futures::channel::oneshot::channel();
        let view = cx.new(|cx| ModelInstall::new(source, model, app, cx));
        let mut answer = Some(answer);
        cx.subscribe(&view, move |shell, view, outcome: &InstallOutcome, cx| {
            if let Some(answer) = answer.take() {
                let _ = answer.send(outcome.clone());
            }
            if shell.install.as_ref() == Some(&view) {
                shell.install = None;
                cx.notify();
            }
        })
        .detach();
        self.install = Some(view);
        cx.notify();
        answered
    }

    /// Serve apps' window requests from `queued` (see [`install::requests`]).
    /// Installing a model shows its download with the app's name; the app
    /// learns the outcome from the engine (`models.info`). A reported
    /// problem is written as `app_error` and asked about.
    pub fn serve_requests(
        &mut self,
        mut queued: futures::channel::mpsc::UnboundedReceiver<(
            String,
            isonomic_app::WindowRequest,
        )>,
        source: Arc<dyn ModelSource>,
        cx: &mut Context<Self>,
    ) {
        use futures::StreamExt as _;
        cx.spawn(async move |shell, cx| {
            while let Some((app, request)) = queued.next().await {
                let served = shell.update(cx, |shell, cx| {
                    if let isonomic_app::WindowRequest::Report(problem) = request {
                        return shell.app_reported(&app, problem, cx);
                    }
                    if let isonomic_app::WindowRequest::InstallModel { model } = request {
                        let name = shell
                            .tabs
                            .iter()
                            .find(|tab| tab.id == app)
                            .map(|tab| tab.title.to_string())
                            .unwrap_or(app);
                        // The app reads the outcome from the engine, not from here.
                        drop(shell.install_model(model, name, Arc::clone(&source), cx));
                    }
                });
                if served.is_err() {
                    break; // the shell is gone
                }
            }
        })
        .detach();
    }

    /// The model install on screen, if any.
    pub fn model_install(&self) -> Option<&Entity<ModelInstall>> {
        self.install.as_ref()
    }

    /// The first-run flow, while it is showing.
    pub fn first_run(&self) -> Option<&Entity<FirstRun>> {
        self.first_run.as_ref()
    }

    /// The Home tab.
    pub fn home(&self) -> &Entity<HomeTab> {
        &self.home
    }

    /// Put keyboard focus where a keyboard user starts: the selected tab's
    /// rail button, or first run while it shows. The window calls it when
    /// it opens.
    pub fn focus_rail(&self, window: &mut Window, cx: &mut Context<Self>) {
        match &self.first_run {
            // First run stands in for the tabs, so focus starts there.
            Some(first_run) => first_run.read(cx).focus_handle().clone().focus(window, cx),
            None => self.tabs[self.selected].focus.focus(window, cx),
        }
    }

    /// Be told the id of the tab on screen when it changes, as problem
    /// reports record it.
    pub fn on_tab_change(&mut self, tell: impl Fn(&str) + 'static) {
        self.on_tab = Some(Box::new(tell));
    }

    /// The rail button of the tab with `id`, for following keyboard focus.
    pub fn rail_focus(&self, id: &str) -> Option<&FocusHandle> {
        Some(&self.tabs.iter().find(|tab| tab.id == id)?.focus)
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
        let icon = manifest.icon.clone();
        let app = apps::tab(registration, host, cx);
        // The rail redraws when the app asks for attention.
        cx.observe(&app, |_, _, cx| cx.notify()).detach();
        // A contained failure wrote a report: ask about it straight away.
        cx.subscribe(&app, |shell, _, _: &apps::Stopped, cx| {
            shell.reports_due(cx)
        })
        .detach();
        if let Some(reports) = &self.reports {
            reports.update(cx, |r, _| r.name_app(&id, &title));
        }
        self.tabs.push(Tab {
            id: id.into(),
            title: title.into(),
            icon: icon.into(),
            focus: cx.focus_handle(),
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

    /// Select the tab of the app with `id` and build it if needed, as a
    /// click on the rail would. `false` if no such app is mounted.
    pub fn open_app(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(index) = self
            .tabs
            .iter()
            .position(|tab| tab.app.is_some() && tab.id == id)
        else {
            return false;
        };
        if index == self.selected {
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
        if let Some(tell) = &self.on_tab {
            tell(&self.tabs[index].id);
        }
        if let Some(app) = &self.tabs[index].app {
            app.update(cx, |app, cx| app.activate(window, cx));
            self.recent.retain(|&i| i != index);
            self.recent.push(index);
            self.enforce_cap(cx);
        }
        cx.notify();
    }
}

/// The icon a manifest names, when Isonomic UI's catalog has it.
fn catalog_icon(name: &str) -> Option<ui::assets::IconName> {
    let path = format!("icons/{name}.svg");
    ui::assets::IconName::ALL
        .iter()
        .copied()
        .find(|icon| icon.path().as_ref() == path)
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(first_run) = &self.first_run {
            return div().id("shell").size_full().child(first_run.clone());
        }
        // Notices wait until first run is over; a dialog opens outside the draw.
        if let Some(reports) = self.reports.clone().filter(|_| self.reports_due) {
            self.reports_due = false;
            window.defer(cx, move |window, cx| {
                reports.update(cx, |r, cx| r.review(window, cx));
            });
        }
        // A rail button that takes keyboard focus is scrolled into view,
        // which GPUI and Isonomic UI don't do on their own.
        let focused = self
            .tabs
            .iter()
            .position(|tab| tab.focus.is_focused(window));
        if focused.is_some() && focused != self.rail_followed {
            if let Some(index) = focused {
                self.rail_scroll.scroll_to_item(index);
                window.refresh();
            }
        }
        self.rail_followed = focused;
        let theme = cx.theme();
        let rail_items: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let selected = index == self.selected;
                let attention = tab
                    .app
                    .as_ref()
                    .map_or(Attention::None, |app| app.read(cx).attention());
                // An icon-only button, named by its tooltip; the tab order
                // follows the rail.
                // The design's collapsed rail: 28 px ghost icon buttons.
                let mut button = Button::new(SharedString::from(format!("rail-button-{}", tab.id)))
                    .ghost()
                    .small()
                    .size(tokens::rail_button())
                    .selected(selected)
                    .tooltip(tab.title.clone())
                    .accessibility_label(tab.title.clone())
                    .track_focus(&tab.focus)
                    .on_click(
                        cx.listener(move |shell, _, window, cx| shell.select(index, window, cx)),
                    );
                button = match catalog_icon(&tab.icon) {
                    Some(icon) => button.icon(Icon::from(icon)),
                    // No such icon: the app's initial stands in.
                    None => button.label(tab.title.chars().take(1).collect::<String>()),
                };
                let marked = match attention {
                    Attention::None => button.into_any_element(),
                    Attention::Dot => div()
                        .id(SharedString::from(format!("attention-{}", tab.id)))
                        .test_support()
                        .child(Badge::new().dot().child(button))
                        .into_any_element(),
                    Attention::Count(n) => div()
                        .id(SharedString::from(format!("attention-{}", tab.id)))
                        .test_support()
                        .child(Badge::new().count(n as usize).child(button))
                        .into_any_element(),
                };
                div()
                    .id(SharedString::from(format!("rail-{}", tab.id)))
                    .test_support()
                    .child(marked)
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
                    .py_3()
                    .px_2()
                    .overflow_y_scroll()
                    .track_scroll(&self.rail_scroll)
                    .bg(theme.sidebar)
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
                    .child(match &self.install {
                        Some(install) => install.clone().into_any_element(),
                        None => self.tabs[self.selected].view.clone().into_any_element(),
                    }),
            )
    }
}
