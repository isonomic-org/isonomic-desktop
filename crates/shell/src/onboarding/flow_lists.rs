// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two list steps' panels: "Turn on local AI models", whose rows are
//! the models the engine says this Mac may run, and "Integrations and
//! connected apps", whose rows are the design's.
//!
//! A model switched on is fetched by "Setting up your workspace". An
//! integration asked for is only kept as a choice: nothing connects yet.

use ui::assets::IconName;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::input::Input;
use ui::component::link::Link;
use ui::component::switch::Switch;
use ui::component::tab::{Tab, TabBar};
use ui::component::{Disableable as _, Icon, Sizable as _, Size};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, Context, Div, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, SharedString, Styled as _,
};

use super::backend::Memory;
use super::flow_bodies::{held, icon, named, note, outlined, text};
use super::logos;
use super::steps;
use super::Onboarding;

/// A row of a list: its picture, its two lines and its control.
struct Row {
    key: usize,
    picture: AnyElement,
    name: SharedString,
    about: SharedString,
    control: AnyElement,
}

/// Words alone where a list would be.
fn instead(says: impl Into<SharedString>) -> Div {
    text(shared::ACTION_LABEL)
        .w_full()
        .py(px(design::EMPTY_PAD))
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .text_color(fourth::faint())
        .child(says.into())
}

fn gib(bytes: u64) -> String {
    format!("{} GB", (bytes as f64 / (1u64 << 30) as f64).round())
}

impl Onboarding {
    /// What the last call that failed said, between a step's two buttons.
    pub(super) fn trouble_line(&self) -> Option<Div> {
        let said = self.trouble.clone()?;
        Some(
            text(design::SMALL)
                .flex_1()
                .min_w_0()
                .px_4()
                .text_center()
                .text_color(ui::onboarding::kit::wrong())
                .child(said),
        )
    }

    /// A switch, which flips when pressed.
    pub(super) fn switch(
        &self,
        step: usize,
        index: usize,
        (on, fixed): (bool, bool),
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        held(
            Switch::new(("onboarding-switch", step * 16 + index))
                .large()
                .checked(on)
                .disabled(fixed)
                .on_click(cx.listener(move |view, _: &bool, _, cx| view.flip(step, index, cx))),
        )
    }

    /// "Connect", or "Connected" once pressed.
    fn connect(&self, index: usize, on: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let id = ("onboarding-connect", index);
        let button = if on {
            Button::new(id)
                .primary()
                .icon(icon(IconName::Check))
                .icon_size(Size::Size(px(design::ICON)))
                .label("Connected")
        } else {
            Button::new(id).outline().label("Connect")
        };
        held(
            button
                .h(px(design::CONNECT))
                .px(px(design::CONNECT_PAD))
                .on_click(cx.listener(move |view, _, _, cx| view.flip(steps::LINKS, index, cx))),
        )
    }

    /// What a list step has in its header, beside and under its heading:
    /// the button beside it, and its tabs with their search box.
    pub(super) fn header_extras(&self, step: usize, cx: &mut Context<Self>) -> Option<(Div, Div)> {
        let (which, add, names): (usize, _, &'static [&'static str]) = match step {
            steps::MODELS => (0, "Import model", &steps::MODEL_TABS),
            steps::LINKS => (1, "Custom integration", &steps::LINK_TABS),
            _ => return None,
        };
        let tabs = TabBar::new(("onboarding-tabs", which))
            .segmented()
            .selected_index(self.tabs[which])
            .children(names.iter().map(|name| Tab::new().label(*name)))
            .on_click(cx.listener(move |view, chosen: &usize, _, cx| {
                view.tabs[which] = *chosen;
                cx.notify();
            }));
        let search = self.search.as_ref().map(|search| {
            div().flex_1().child(
                Input::new(&search[which])
                    .prefix(icon(IconName::Search))
                    .min_h(px(design::BUTTON)),
            )
        });
        let filters = div()
            .w_full()
            .flex()
            .items_center()
            .gap_3()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(div().flex_none().child(tabs))
            .children(search);
        Some((
            held(outlined(
                ("onboarding-add", which),
                add,
                Some(IconName::Plus),
            )),
            filters,
        ))
    }

    /// What is in list `which`'s search box.
    fn sought(&self, which: usize, cx: &mut Context<Self>) -> SharedString {
        self.search
            .as_ref()
            .map_or_else(Default::default, |search| search[which].read(cx).value())
    }

    /// The rows, or what is said when there are none.
    fn listed(step: usize, rows: Vec<Row>, sought: &str, nothing: &'static str) -> AnyElement {
        if rows.is_empty() {
            let says = if sought.trim().is_empty() {
                nothing.to_owned()
            } else {
                format!("Nothing matches \u{201c}{}\u{201d}.", sought.trim())
            };
            return instead(says).into_any_element();
        }
        let rows = rows.into_iter().map(|row| {
            div()
                .w_full()
                .h(px(design::ROW))
                .flex()
                .items_center()
                .gap(px(design::LOGO_GAP))
                .border_b_1()
                .border_color(page::hairline())
                .child(row.picture)
                .child(named(row.name, row.about))
                .child(
                    Link::new(("onboarding-learn", step * 16 + row.key))
                        .flex_none()
                        .text_size(px(shared::ACTION_LABEL.0))
                        .line_height(px(shared::ACTION_LABEL.1))
                        .text_color(shared::quiet())
                        // Underlined only when pointed at, as the design has it.
                        .text_decoration_0()
                        .child("Learn more"),
                )
                .child(row.control)
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    }

    /// A model's mark: its maker's, or a chip where the design has none.
    fn model_mark(name: &str) -> AnyElement {
        match logos::model(name) {
            Some(logo) => img(logo)
                .flex_none()
                .size(px(design::LOGO))
                .into_any_element(),
            None => div()
                .flex_none()
                .size(px(design::LOGO))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(fourth::CARD_CORNER))
                .border_1()
                .border_color(page::hairline())
                .bg(design::field())
                .text_color(page::ink())
                .child(Icon::from(IconName::Chip).size(px(design::PASSKEY_ICON)))
                .into_any_element(),
        }
    }

    /// "Turn on local AI models": the models this Mac may run.
    pub(super) fn models_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(catalog) = &self.catalog else {
            return match &self.catalog_trouble {
                Some(said) => instead(said.clone())
                    .text_color(ui::onboarding::kit::wrong())
                    .child(held(
                        outlined("onboarding-models-again", "Try again", None)
                            .on_click(cx.listener(|view, _, _, cx| view.load_models(cx))),
                    ))
                    .into_any_element(),
                None => {
                    instead("Looking for the models this Mac can run\u{2026}").into_any_element()
                }
            };
        };
        let sought = self.sought(0, cx);
        let short = catalog.memory != Memory::Enough;
        let rows = catalog
            .models
            .iter()
            .enumerate()
            .filter(|(_, model)| {
                steps::listed(
                    model.kind,
                    self.tabs[0],
                    &sought,
                    (&model.name, &model.about),
                )
            })
            .map(|(index, model)| Row {
                key: index,
                picture: Self::model_mark(&model.name),
                name: model.name.clone().into(),
                about: model.about.clone().into(),
                control: self
                    .switch(
                        steps::MODELS,
                        index,
                        (self.models[index], short || model.installed),
                        cx,
                    )
                    .into_any_element(),
            })
            .collect();
        let list = Self::listed(steps::MODELS, rows, &sought, "No models here yet.");
        let Memory::Short { has, needs } = catalog.memory else {
            return list;
        };
        div()
            .w_full()
            .pt_4()
            .flex()
            .flex_col()
            .gap_2()
            .child(note(
                "onboarding-memory-note",
                (
                    "This Mac can\u{2019}t run local models yet",
                    format!(
                        "It has {} of memory. Isonomic needs at least {} to run its local models.",
                        gib(has),
                        gib(needs)
                    ),
                ),
            ))
            .child(list)
            .into_any_element()
    }

    /// "Integrations and connected apps": the design's rows.
    pub(super) fn links_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let sought = self.sought(1, cx);
        let rows = steps::LINK_LIST
            .iter()
            .enumerate()
            .filter(|(index, (name, about))| {
                steps::listed(
                    steps::LINK_KINDS[*index],
                    self.tabs[1],
                    &sought,
                    (name, about),
                )
            })
            .map(|(index, (name, about))| Row {
                key: index,
                picture: img(logos::link(index))
                    .flex_none()
                    .size(px(design::LOGO))
                    .into_any_element(),
                name: (*name).into(),
                about: (*about).into(),
                control: self
                    .connect(index, self.links[index], cx)
                    .into_any_element(),
            })
            .collect();
        Self::listed(steps::LINKS, rows, &sought, "No integrations here yet.")
    }
}
