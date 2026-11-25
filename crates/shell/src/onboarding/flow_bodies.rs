// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is in each later step's panel, under its heading. Every control
//! is Isonomic UI's: its buttons, switches, tabs, inputs, checkbox, tags
//! and alerts. A first pass: the switches, the tabs, the checkbox and the
//! connect buttons flip, and nothing else here does anything yet.

use ui::assets::IconName;
use ui::component::alert::Alert;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::checkbox::Checkbox;
use ui::component::input::Input;
use ui::component::link::Link;
use ui::component::switch::Switch;
use ui::component::tab::{Tab, TabBar};
use ui::component::tag::Tag;
use ui::component::{Icon, Sizable as _, Size};
use ui::onboarding::kit::{self as grid};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, Context, Div, ElementId, FontWeight, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, SharedString, Styled as _,
};

use super::logos;
use super::steps;
use super::Onboarding;

pub(super) fn card() -> Div {
    let (pad_x, pad_y) = fourth::CARD_PAD;
    // The design's padding has the border in it.
    div()
        .w_full()
        .px(px(pad_x - 1.))
        .py(px(pad_y - 1.))
        .rounded(px(fourth::CARD_CORNER))
        .border_1()
        .border_color(page::hairline())
        .flex()
        .flex_col()
}

pub(super) fn text(size: (f32, f32)) -> Div {
    div().text_size(px(size.0)).line_height(px(size.1))
}

pub(super) fn caption(words: &'static str) -> Div {
    text(fourth::CAPTION)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(fourth::faint())
        .child(words)
}

/// One of the design's icons, at the size it has in a button.
pub(super) fn icon(name: IconName) -> Icon {
    Icon::from(name).size(px(design::ICON))
}

/// Something pressed: a press on it is a press, not the start of moving
/// the window.
pub(super) fn held(control: impl IntoElement) -> Div {
    div()
        .flex_none()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .child(control)
}

/// An outlined button, as tall as the design's, with an icon before its
/// label if it has one.
pub(super) fn outlined(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    before: Option<IconName>,
) -> Button {
    let button = Button::new(id)
        .outline()
        .label(label)
        .h(px(design::BUTTON))
        .px(px(design::BUTTON_PAD));
    match before {
        Some(name) => button
            .icon(icon(name))
            .icon_size(Size::Size(px(design::ICON))),
        None => button,
    }
}

/// A note with its title, as the design's cards with an "i" have it.
pub(super) fn note(id: &'static str, (title, words): (&'static str, &'static str)) -> Alert {
    Alert::new(id, words)
        .title(title)
        .bg(design::field())
        .text_size(px(design::SMALL.0))
}

/// A name with a line about it under it.
fn named(name: &'static str, about: &'static str) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            text(fourth::ROW_TITLE)
                .font_weight(FontWeight::MEDIUM)
                .text_color(page::ink())
                .child(name),
        )
        .child(text(design::SMALL).text_color(fourth::faint()).child(about))
}

/// A row of the kit's words, four across.
pub(super) fn word_rows(cells: Vec<AnyElement>) -> Vec<Div> {
    let mut rows = Vec::new();
    let mut cells = cells.into_iter().peekable();
    while cells.peek().is_some() {
        rows.push(
            div()
                .w_full()
                .flex()
                .gap(px(grid::CELL_GAP))
                .children(cells.by_ref().take(grid::ACROSS)),
        );
    }
    rows
}

impl Onboarding {
    /// A switch, which flips when pressed.
    fn switch(
        &self,
        step: usize,
        index: usize,
        on: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        held(
            Switch::new(("onboarding-switch", step * 16 + index))
                .large()
                .checked(on)
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

    /// A list of things to switch on or connect.
    fn list(
        &self,
        step: usize,
        rows: &'static [(&'static str, &'static str, bool)],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = rows.iter().enumerate().map(|(index, (name, about, _))| {
            let control = if step == steps::MODELS {
                self.switch(step, index, self.models[index], cx)
                    .into_any_element()
            } else {
                self.connect(index, self.links[index], cx)
                    .into_any_element()
            };
            div()
                .w_full()
                .h(px(design::ROW))
                .flex()
                .items_center()
                .gap(px(design::LOGO_GAP))
                .border_b_1()
                .border_color(page::hairline())
                .child(
                    img(logos::logo(step, index))
                        .flex_none()
                        .size(px(design::LOGO)),
                )
                .child(named(name, about))
                .child(
                    Link::new(("onboarding-learn", step * 16 + index))
                        .flex_none()
                        .text_size(px(shared::ACTION_LABEL.0))
                        .line_height(px(shared::ACTION_LABEL.1))
                        .text_color(shared::quiet())
                        .child("Learn more"),
                )
                .child(control)
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .children(rows)
            .into_any_element()
    }

    fn created(&self) -> AnyElement {
        let root = steps::ROOT.iter().enumerate().map(|(index, (what, how))| {
            let row = div()
                .w_full()
                .py(px(design::ROOT_ROW_PAD))
                .flex()
                .justify_between()
                .border_color(page::hairline())
                .child(
                    text(fourth::ROW_TEXT)
                        .text_color(shared::quiet())
                        .child(*what),
                )
                .child(
                    text(fourth::ROW_TEXT)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(page::ink())
                        .child(*how),
                );
            if index == 0 {
                row
            } else {
                row.border_t_1()
            }
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(design::CARD_GAP))
            .child(
                card()
                    .bg(design::field())
                    .child(caption(
                        "SMART ACCOUNT (ERC-4337) \u{b7} BASE + ISONOMIC L2",
                    ))
                    .child(
                        // The address, with its button in its box.
                        div()
                            .w_full()
                            .pl_3()
                            .pr(px(design::ADDRESS_PAD))
                            .py(px(design::ADDRESS_PAD))
                            .flex()
                            .items_center()
                            .gap_2()
                            .rounded(px(shared::ACTION_CORNER))
                            .border_1()
                            .border_color(shared::outline())
                            .bg(page::page())
                            .child(
                                text(fourth::ROW_TEXT)
                                    .flex_1()
                                    .font_family(shared::MONO)
                                    .text_color(page::ink())
                                    .child(steps::ADDRESS),
                            )
                            .child(held(outlined("onboarding-copy", "Copy", None))),
                    )
                    .child(
                        text(design::SMALL)
                            .max_w(px(design::NOTE_WIDE))
                            .text_color(fourth::faint())
                            .child(steps::ADDRESS_NOTE),
                    ),
            )
            .child(
                card()
                    .child(caption("THIS ADDRESS IS THE WORKSPACE ROOT"))
                    .children(root),
            )
            .child(
                card().child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
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
                                .child(
                                    Icon::from(IconName::FingerprintRecognition)
                                        .size(px(design::PASSKEY_ICON)),
                                ),
                        )
                        .child(named(
                            "This Mac \u{b7} Touch ID",
                            "Synced through iCloud Keychain \u{b7} PRF supported",
                        ))
                        .child(
                            Tag::custom(design::good(), design::good_ink(), design::good())
                                .gap_1p5()
                                .child(div().size_1p5().rounded_full().bg(design::good_ink()))
                                .child("Active"),
                        ),
                ),
            )
            .child(div().flex().child(held(outlined(
                "onboarding-add-passkey",
                "Add another passkey",
                Some(IconName::AddAlt),
            ))))
            .into_any_element()
    }

    fn kit(&self, cx: &mut Context<Self>) -> AnyElement {
        let cells = steps::WORDS
            .iter()
            .enumerate()
            .map(|(index, word)| {
                div()
                    .flex_1()
                    .h(px(grid::CELL))
                    .px(px(fourth::ROW_PAD))
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(px(shared::ACTION_CORNER))
                    .border_1()
                    .border_color(shared::outline())
                    .font_family(shared::SERIF)
                    .child(
                        text(fourth::ROW_TEXT)
                            .text_color(fourth::faint())
                            .child(format!("{}.", index + 1)),
                    )
                    .child(text(fourth::ROW_TITLE).text_color(page::ink()).child(*word))
                    .into_any_element()
            })
            .collect();
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(caption(
                "ISONOMIC RECOVERY KIT \u{b7} WORKSPACE MASTER KEY \u{b7} 24 WORDS",
            ))
            .children(word_rows(cells))
            .child(
                text(design::SMALL)
                    .text_color(fourth::faint())
                    .child(steps::KIT_NOTE),
            )
            .child(
                div()
                    .w_full()
                    .py_1()
                    .flex()
                    .items_center()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Checkbox::new("onboarding-stored")
                            .checked(self.stored)
                            .label("I\u{2019}ve stored this somewhere safe, off this machine.")
                            .on_click(cx.listener(|view, stored: &bool, _, cx| {
                                view.stored = *stored;
                                cx.notify();
                            })),
                    )
                    .child(div().flex_1())
                    .child(held(outlined(
                        "onboarding-print",
                        "Print",
                        Some(IconName::Printer),
                    )))
                    .child(held(outlined(
                        "onboarding-pdf",
                        "Save as PDF",
                        Some(IconName::Download),
                    ))),
            )
            .child(note("onboarding-kit-note", steps::KIT_WARNING))
            .into_any_element()
    }

    fn finish(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<_> = steps::RESIDENCY
            .iter()
            .enumerate()
            .map(|(index, (name, about, _))| {
                let row = div()
                    .w_full()
                    .py_4()
                    .flex()
                    .items_start()
                    .gap_3()
                    .border_color(page::hairline())
                    .child(named(name, about).max_w(px(design::SETTING_WIDE)))
                    .child(div().flex_1())
                    .child(self.switch(steps::FINISH, index, self.residency[index], cx));
                if index == 0 {
                    row
                } else {
                    row.border_t_1()
                }
            })
            .collect();
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_5()
            .child(card().py_1().children(rows))
            .child(note("onboarding-finish-note", steps::AFTER))
            .into_any_element()
    }

    /// The body of step `index`'s panel.
    pub(super) fn body(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        match index {
            steps::CREATED => self.created(),
            steps::KIT => self.kit(cx),
            steps::CONFIRM => self.confirm(),
            steps::MODELS => self.list(index, &steps::MODEL_LIST, cx),
            steps::LINKS => self.list(index, &steps::LINK_LIST, cx),
            _ => self.finish(cx),
        }
    }
}
