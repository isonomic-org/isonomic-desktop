// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The two list steps' panels: "Turn on local AI models", whose rows are
//! the engine's catalogue of local models, each with its Recommended or
//! Not recommended pill; and "Integrations and connected apps", whose
//! rows are the connector catalogue (`crate::connectors`).
//!
//! A model or a connector switched on is fetched or connected by
//! "Setting up your workspace". A connector that takes a token asks for
//! it in its row before its switch goes on.

use ui::assets::IconName;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::input::Input;
use ui::component::link::Link;
use ui::component::switch::Switch;
use ui::component::tab::{Tab, TabBar};
use ui::component::{Disableable as _, Sizable as _, Size};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::terms::{self as shared};
use ui::{
    div, px, AnyElement, Context, Div, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::backend::Memory;
use super::flow_bodies::{held, icon, note, outlined, text};
use super::steps;
use super::Onboarding;
use crate::connectors::{self, CONNECTORS};
use crate::lists::{self, named};
use crate::logos;

/// A row of a list: its picture, its two lines, what is beside them and
/// its control.
struct Row {
    key: usize,
    picture: AnyElement,
    name: SharedString,
    about: SharedString,
    beside: Vec<AnyElement>,
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

    /// "Connect", or "Connected" once pressed, as the design has it.
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
        let (which, add, names): (usize, _, Vec<SharedString>) = match step {
            steps::MODELS => (0, "Import model", self.model_tabs()),
            steps::LINKS => (1, "Custom integration", self.link_tabs()),
            _ => return None,
        };
        let tabs = TabBar::new(("onboarding-tabs", which))
            .segmented()
            .selected_index(self.tabs[which])
            .children(names.into_iter().map(|name| Tab::new().label(name)))
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
            let learn = Link::new(("onboarding-learn", step * 64 + row.key))
                .flex_none()
                .text_size(px(shared::ACTION_LABEL.0))
                .line_height(px(shared::ACTION_LABEL.1))
                .text_color(shared::quiet())
                // Underlined only when pointed at, as the design has it.
                .text_decoration_0()
                .child("Learn more");
            let mut beside = row.beside;
            beside.push(learn.into_any_element());
            lists::row(row.picture, named(row.name, row.about), beside, row.control)
        });
        div()
            .id(("onboarding-list", step))
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    }

    /// The models' tabs, from the catalogue: "All models" until it is here.
    pub(super) fn model_tabs(&self) -> Vec<SharedString> {
        self.catalog.as_ref().map_or_else(
            || vec![crate::workspace::ALL_MODELS.into()],
            |catalog| catalog.tabs().into_iter().map(Into::into).collect(),
        )
    }

    fn link_tabs(&self) -> Vec<SharedString> {
        connectors::tabs().into_iter().map(Into::into).collect()
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
            .filter(|(_, model)| catalog.listed(model, self.tabs[0], &sought))
            .map(|(index, model)| Row {
                key: index,
                picture: lists::mark(
                    logos::model(&model.name, model.publisher.as_deref()),
                    &model.name,
                ),
                name: model.name.clone().into(),
                about: model.line().into(),
                beside: vec![lists::fit_pill(("onboarding-fit", index), model)],
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

    /// "Integrations and connected apps": the connector catalogue. One
    /// that is coming soon has its tag where "Connect" would be; one that
    /// asks for a token or an address shows its field when "Connect" is
    /// pressed, and is connected once something is typed there.
    pub(super) fn links_list(&self, cx: &mut Context<Self>) -> AnyElement {
        let sought = self.sought(1, cx);
        let rows = CONNECTORS
            .iter()
            .enumerate()
            .filter(|(_, connector)| connectors::listed(connector, self.tabs[1], &sought))
            .map(|(index, connector)| {
                // The field shows once "Connect" is pressed on a row that
                // needs something typed; the row is the design's until then.
                let field = self
                    .field(index)
                    .filter(|_| connector.available && self.asking_for == Some(index));
                let beside = field
                    .map(|field| {
                        held(
                            div()
                                .w(px(design::ROW_FIELD))
                                .child(Input::new(field).min_h(px(design::CONNECT))),
                        )
                        .into_any_element()
                    })
                    .into_iter()
                    .collect();
                let control = if connector.available {
                    let on = self.links.get(index).copied().unwrap_or(false);
                    self.connect(index, on, cx).into_any_element()
                } else {
                    lists::coming_soon()
                };
                Row {
                    key: index,
                    picture: lists::mark(connector.logo.and_then(logos::named), connector.name),
                    name: connector.name.into(),
                    about: connector.about.into(),
                    beside,
                    control,
                }
            })
            .collect();
        Self::listed(steps::LINKS, rows, &sought, "No integrations here yet.")
    }
}
