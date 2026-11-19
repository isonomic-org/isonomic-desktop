// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is in each later step's panel, under its heading. A first pass:
//! the switches and the connect buttons flip, and nothing else here does
//! anything yet.

use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
};

use super::logos;
use super::steps;
use super::Onboarding;

fn card() -> Div {
    let (pad_x, pad_y) = fourth::CARD_PAD;
    div()
        .w_full()
        .px(px(pad_x))
        .py(px(pad_y))
        .rounded(px(fourth::CARD_CORNER))
        .border_1()
        .border_color(page::hairline())
        .flex()
        .flex_col()
}

fn text(size: (f32, f32)) -> Div {
    div().text_size(px(size.0)).line_height(px(size.1))
}

fn caption(words: &'static str) -> Div {
    text(fourth::CAPTION)
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(fourth::faint())
        .child(words)
}

/// A small button that does nothing yet.
fn aside(label: &'static str) -> Div {
    let (_, _, above) = shared::ACTION_PAD;
    div()
        .flex_none()
        .px(px(fourth::ASIDE_PAD))
        .py(px(above))
        .rounded(px(shared::ACTION_CORNER))
        .border_1()
        .border_color(shared::outline())
        .bg(page::page())
        .shadow(shared::action_shadow())
        .font_weight(FontWeight::MEDIUM)
        .text_color(shared::body())
        .child(text(shared::ACTION_LABEL).child(label))
}

/// A name with a line about it under it.
fn named(name: &'static str, about: &'static str) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            text(fourth::ROW_TITLE)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(page::ink())
                .child(name),
        )
        .child(text(design::SMALL).text_color(fourth::faint()).child(about))
}

/// The tabs over a list, the first one chosen, and a search box.
fn tabs(names: &'static [&'static str]) -> Div {
    let chosen = names.iter().enumerate().map(|(index, name)| {
        let tab = text(fourth::ROW_TEXT)
            .px(px(10.))
            .py(px(5.))
            .rounded(px(shared::ACTION_CORNER))
            .text_color(shared::quiet())
            .child(*name);
        if index == 0 {
            tab.bg(design::wash())
                .font_weight(FontWeight::MEDIUM)
                .text_color(page::ink())
        } else {
            tab
        }
    });
    div()
        .w_full()
        .h(px(design::TABS))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_none()
                .h_full()
                .px(px(4.))
                .flex()
                .items_center()
                .gap(px(2.))
                .rounded(px(fourth::CARD_CORNER))
                .border_1()
                .border_color(page::hairline())
                .children(chosen),
        )
        .child(
            text(fourth::ROW_TEXT)
                .flex_1()
                .h_full()
                .px(px(12.))
                .flex()
                .items_center()
                .rounded(px(fourth::CARD_CORNER))
                .border_1()
                .border_color(shared::outline())
                .text_color(fourth::faint())
                .child("Search"),
        )
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
        let (wide, tall, knob) = design::SWITCH;
        let inset = (tall - knob) / 2.;
        div()
            .id(("onboarding-switch", step * 16 + index))
            .flex_none()
            .relative()
            .w(px(wide))
            .h(px(tall))
            .rounded(px(tall / 2.))
            .bg(if on { page::ink() } else { shared::outline() })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |view, _, _, cx| view.flip(step, index, cx)))
            .child(
                div()
                    .absolute()
                    .top(px(inset))
                    .left(px(if on { wide - knob - inset } else { inset }))
                    .size(px(knob))
                    .rounded(px(knob / 2.))
                    .bg(page::page()),
            )
    }

    /// "Connect", or "Connected" once pressed.
    fn connect(&self, index: usize, on: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let button = aside(if on { "Connected" } else { "Connect" })
            .id(("onboarding-connect", index))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |view, _, _, cx| view.flip(steps::LINKS, index, cx)));
        if on {
            button
                .bg(page::ink())
                .border_color(page::ink())
                .text_color(page::page())
        } else {
            button
        }
    }

    /// A list of things to switch on or connect, under its tabs.
    fn list(
        &self,
        step: usize,
        names: &'static [&'static str],
        rows: &'static [(&'static str, &'static str, bool)],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = rows.iter().enumerate().map(|(index, (name, about, _))| {
            let on = if step == steps::MODELS {
                self.models[index]
            } else {
                self.links[index]
            };
            let control = if step == steps::MODELS {
                self.switch(step, index, on, cx).into_any_element()
            } else {
                self.connect(index, on, cx).into_any_element()
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
                    text(fourth::ROW_TEXT)
                        .flex_none()
                        .text_color(shared::quiet())
                        .child("Learn more"),
                )
                .child(control)
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(tabs(names))
            .child(div().h(px(8.)))
            .children(rows)
            .into_any_element()
    }

    fn created(&self) -> AnyElement {
        let root = steps::ROOT.iter().enumerate().map(|(index, (what, how))| {
            let row = div()
                .w_full()
                .py(px(9.))
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
                        .font_weight(FontWeight::SEMIBOLD)
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
            .gap(px(20.))
            .child(
                card()
                    .gap(px(8.))
                    .child(caption(
                        "SMART ACCOUNT (ERC-4337) \u{b7} BASE + ISONOMIC L2",
                    ))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                text(fourth::ROW_TEXT)
                                    .flex_1()
                                    .px(px(12.))
                                    .py(px(8.))
                                    .rounded(px(shared::ACTION_CORNER))
                                    .border_1()
                                    .border_color(shared::outline())
                                    .font_family(shared::MONO)
                                    .text_color(page::ink())
                                    .child(steps::ADDRESS),
                            )
                            .child(aside("Copy")),
                    )
                    .child(
                        text(design::SMALL)
                            .max_w(px(520.))
                            .text_color(fourth::faint())
                            .child(steps::ADDRESS_NOTE),
                    ),
            )
            .child(
                card()
                    .child(caption("THIS ADDRESS IS THE WORKSPACE ROOT"))
                    .child(div().h(px(6.)))
                    .children(root),
            )
            .child(
                card().child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(named(
                            "This Mac \u{b7} Touch ID",
                            "Synced through iCloud Keychain \u{b7} PRF supported",
                        ))
                        .child(
                            text(design::SMALL)
                                .flex_none()
                                .px(px(8.))
                                .rounded(px(shared::ACTION_CORNER))
                                .bg(design::good())
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(design::good_ink())
                                .child("Active"),
                        ),
                ),
            )
            .child(div().flex().child(aside("Add another passkey")))
            .into_any_element()
    }

    fn kit(&self) -> AnyElement {
        let (title, words) = steps::KIT_WARNING;
        let cells = steps::WORDS.iter().enumerate().map(|(index, word)| {
            div()
                .w(px(190.))
                .h(px(design::WORD))
                .px(px(10.))
                .flex()
                .items_center()
                .gap(px(8.))
                .rounded(px(4.))
                .border_1()
                .border_color(shared::outline())
                .child(
                    text(fourth::ROW_TEXT)
                        .text_color(fourth::faint())
                        .child(format!("{}.", index + 1)),
                )
                .child(
                    text(fourth::ROW_TITLE)
                        .font_family(shared::SERIF)
                        .text_color(page::ink())
                        .child(*word),
                )
        });
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .w_full()
                    .px(px(20.))
                    .py(px(14.))
                    .rounded(px(fourth::CARD_CORNER))
                    .bg(design::warning())
                    .text_color(design::warning_ink())
                    .child(
                        text(fourth::ROW_TEXT)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(title),
                    )
                    .child(text(design::SMALL).child(words)),
            )
            .child(caption(
                "ISONOMIC RECOVERY KIT \u{b7} WORKSPACE MASTER KEY \u{b7} 24 WORDS",
            ))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .children(cells),
            )
            .child(
                text(design::SMALL)
                    .text_color(fourth::faint())
                    .child(steps::KIT_NOTE),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().size(px(14.)).rounded(px(3.)).bg(page::ink()))
                    .child(
                        text(fourth::ROW_TEXT)
                            .text_color(shared::body())
                            .child("I\u{2019}ve stored this somewhere safe, off this machine."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(aside("Print"))
                    .child(aside("Save as PDF")),
            )
            .into_any_element()
    }

    fn confirm(&self) -> AnyElement {
        let (wide, tall) = design::FIELD;
        let fields = steps::ASKED.iter().map(|place| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    text(fourth::ROW_TEXT)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(shared::body())
                        .child(format!("Word {place}")),
                )
                .child(
                    text(fourth::ROW_TITLE)
                        .w(px(wide))
                        .h(px(tall))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .rounded(px(shared::ACTION_CORNER))
                        .border_1()
                        .border_color(shared::outline())
                        .text_color(page::ink())
                        .child(steps::WORDS[place - 1]),
                )
        });
        div()
            .flex()
            .flex_col()
            .gap(px(20.))
            .children(fields)
            .into_any_element()
    }

    fn finish(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<_> = steps::RESIDENCY
            .iter()
            .enumerate()
            .map(|(index, (name, about, _))| {
                let row = div()
                    .w_full()
                    .py(px(16.))
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .border_color(page::hairline())
                    .child(named(name, about).max_w(px(420.)))
                    .child(div().flex_1())
                    .child(self.switch(steps::FINISH, index, self.residency[index], cx));
                if index == 0 {
                    row
                } else {
                    row.border_t_1()
                }
            })
            .collect();
        let (title, words) = steps::AFTER;
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(card().py(px(4.)).children(rows))
            .child(
                div()
                    .w_full()
                    .px(px(20.))
                    .py(px(16.))
                    .rounded(px(fourth::CARD_CORNER))
                    .bg(design::wash())
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        text(fourth::ROW_TITLE)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(page::ink())
                            .child(title),
                    )
                    .child(
                        text(fourth::ROW_TEXT)
                            .max_w(px(520.))
                            .text_color(shared::quiet())
                            .child(words),
                    ),
            )
            .into_any_element()
    }

    /// The body of step `index`'s panel.
    pub(super) fn body(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        match index {
            steps::CREATED => self.created(),
            steps::KIT => self.kit(),
            steps::CONFIRM => self.confirm(),
            steps::MODELS => self.list(index, &steps::MODEL_TABS, &steps::MODEL_LIST, cx),
            steps::LINKS => self.list(index, &steps::LINK_TABS, &steps::LINK_LIST, cx),
            _ => self.finish(cx),
        }
    }
}
