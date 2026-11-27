// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings in the window: a gear at the foot of the rail, and the pages
//! in the place of the tabs until a tab is chosen again.
//!
//! The window offers Settings only when it is told to (`offer_settings`):
//! the pages hold the design's sample contents until they are connected.

use std::sync::Arc;

use isonomic_app::SettingsStore;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::{Selectable as _, Sizable as _};
use ui::{
    div, tokens, AnyElement, AppContext as _, Context, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Styled as _, TestSupportExt as _, Window,
};

use super::{marks, Settings};
use crate::Shell;

/// The window's hold on Settings.
#[derive(Default)]
pub struct Slot {
    store: Option<Arc<dyn SettingsStore>>,
    view: Option<Entity<Settings>>,
    /// Whether the pages stand in the tabs' place.
    pub(crate) shown: bool,
}

impl Shell {
    /// Put the gear on the rail: Settings will keep its values in `store`.
    pub fn offer_settings(&mut self, store: Arc<dyn SettingsStore>, cx: &mut Context<Self>) {
        self.settings.store = Some(store);
        cx.notify();
    }

    /// Show Settings in the place of the tabs, where it has been offered.
    pub fn show_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(store) = self.settings.store.clone() else {
            return;
        };
        if self.settings.view.is_none() {
            self.settings.view = Some(cx.new(|cx| Settings::new(store, window, cx)));
        }
        self.settings.shown = true;
        cx.notify();
    }

    /// Settings, while it is on show.
    pub fn settings(&self) -> Option<&Entity<Settings>> {
        self.settings.view.as_ref().filter(|_| self.settings.shown)
    }

    /// The foot of the rail, where Settings has been offered: the space
    /// that pushes it down, and the gear.
    pub(crate) fn rail_foot(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        if self.settings.store.is_none() {
            return Vec::new();
        }
        let gear = Button::new("rail-button-settings")
            .ghost()
            .small()
            .size(tokens::rail_button())
            .selected(self.settings.shown)
            .icon(marks::icon("settings"))
            .tooltip("Settings")
            .accessibility_label("Settings")
            .on_click(cx.listener(|shell, _, window, cx| shell.show_settings(window, cx)));
        let gear = div().id("rail-settings").test_support().child(gear);
        vec![div().flex_1().into_any_element(), gear.into_any_element()]
    }

    /// What stands beside the rail: Settings, or the chosen tab.
    pub(crate) fn shown(&self) -> AnyElement {
        match self.settings() {
            Some(settings) => settings.clone().into_any_element(),
            None => self.tabs[self.selected].view.clone().into_any_element(),
        }
    }
}
