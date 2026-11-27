// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings: the design's Settings 0.3, every page, tab, sub-page and
//! dialog of it.
//!
//! The pages are drawn from files (`spec`), so a page is changed in its
//! file. What a switch, a choice or a segment is set to is kept in the
//! Capsule's settings; nothing else on these pages reaches the engine yet,
//! and a button with nothing behind it says so.

mod controls;
mod dialog;
mod figures;
pub mod host;
mod marks;
mod page;
mod row;
mod sidebar;
pub mod spec;
mod specs;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use isonomic_app::SettingsStore;
use serde_json::{Map, Value};
use ui::base::{h_flex, v_flex};
use ui::component::input::{InputEvent, InputState, TextareaState};
use ui::component::notification::Notification;
use ui::component::slider::SliderState;
use ui::component::theme::ActiveTheme as _;
use ui::component::WindowExt as _;
use ui::settings as sizes;
use ui::{
    div, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, ScrollHandle, SharedString, StatefulInteractiveElement as _,
    Styled as _, TestSupportExt as _, Window,
};

/// The Capsule's settings keep the pages' values under this app and key.
pub const APP: &str = "isonomic";
pub const KEY: &str = "settings";

/// Said when a button has nothing behind it yet.
pub const NOT_YET: &str = "This is not connected yet.";

/// The page Settings opens on.
const FIRST: &str = "general";

pub struct Settings {
    store: Arc<dyn SettingsStore>,
    /// The page, tab or sub-page on show.
    page: String,
    /// What each switch, choice and segment is set to, by its key.
    values: Map<String, Value>,
    /// Rows taken off their lists since Settings opened.
    gone: HashSet<String>,
    inputs: HashMap<String, Entity<InputState>>,
    areas: HashMap<String, Entity<TextareaState>>,
    sliders: HashMap<String, Entity<SliderState>>,
    /// The sidebar's search.
    search: Entity<InputState>,
    scroll: ScrollHandle,
}

impl Settings {
    /// Settings on its first page, with what `store` has kept.
    pub fn new(store: Arc<dyn SettingsStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        cx.subscribe(&search, |_, _, _: &InputEvent, cx| cx.notify())
            .detach();
        // The store may read a file: off the foreground thread.
        let read = {
            let store = Arc::clone(&store);
            cx.background_spawn(async move { store.get(APP, KEY) })
        };
        cx.spawn(async move |this, cx| {
            if let Some(Value::Object(kept)) = read.await {
                let _ = this.update(cx, |this, cx| {
                    for (key, value) in kept {
                        this.values.entry(key).or_insert(value);
                    }
                    cx.notify();
                });
            }
        })
        .detach();
        Self {
            store,
            page: FIRST.into(),
            values: Map::new(),
            gone: HashSet::new(),
            inputs: HashMap::new(),
            areas: HashMap::new(),
            sliders: HashMap::new(),
            search,
            scroll: ScrollHandle::new(),
        }
    }

    /// The name of the page, tab or sub-page on show.
    pub fn page(&self) -> &str {
        &self.page
    }

    /// Show the page, tab or sub-page called `id`, from its top.
    pub fn go(&mut self, id: &str, cx: &mut Context<Self>) {
        if spec::spec(id).is_none() || id == self.page {
            return;
        }
        self.page = id.to_string();
        self.scroll = ScrollHandle::new();
        cx.notify();
    }

    /// What the control called `key` is set to, when it has been changed.
    pub fn value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Set the control called `key` and keep it in the Capsule's settings.
    pub fn set(&mut self, key: String, value: Value, cx: &mut Context<Self>) {
        self.values.insert(key, value);
        let (store, kept) = (Arc::clone(&self.store), Value::Object(self.values.clone()));
        cx.background_spawn(async move { store.set(APP, KEY, kept) })
            .detach();
        cx.notify();
    }

    /// The button `label` of `row` on the page or dialog `sid` was pressed:
    /// open what the design gives it, or say that nothing is behind it yet.
    pub fn press(
        &mut self,
        sid: &str,
        row: &str,
        label: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match spec::link(sid, row, label) {
            Some(target) => self.follow(target, window, cx),
            None => window.push_notification(Notification::info(NOT_YET), cx),
        }
    }

    /// Go where a link leads: `d:` and a dialog's name, or `p:` and a
    /// page's.
    pub(crate) fn follow(
        &mut self,
        target: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target.split_once(':') {
            Some(("d", dialog)) => self.open(dialog, window, cx),
            Some(("p", page)) => {
                window.close_all_dialogs(cx);
                self.go(page, cx);
            }
            _ => {}
        }
    }

    /// Open the dialog called `id`.
    pub fn open_dialog(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = spec::named(id) {
            self.open(id, window, cx);
        }
    }

    /// Take `row` of `sid` off its list until Settings is opened again.
    fn remove(&mut self, sid: &str, row: &str, cx: &mut Context<Self>) {
        self.gone.insert(format!("{sid}|{row}"));
        cx.notify();
    }
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.page.clone();
        let blocks = spec::spec(&page)
            .map(|spec| self.page_view(spec, window, cx))
            .unwrap_or_default();
        let sidebar = self.sidebar(cx);
        let background = cx.theme().background;
        h_flex()
            .id("settings")
            .size_full()
            .items_start()
            .bg(background)
            .child(sidebar)
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(
                        h_flex().w_full().justify_center().px_6().py_10().child(
                            v_flex()
                                .id(SharedString::from(format!("settings-page-{page}")))
                                .test_support()
                                .w(sizes::column_width())
                                .max_w_full()
                                .children(blocks),
                        ),
                    ),
            )
    }
}
