// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is in each later step's panel, under its heading. Every control
//! is Isonomic UI's: its buttons, switches, tabs, inputs, checkbox, tags
//! and alerts. What they show and keep is the backend's (`calls`); the two
//! lists are in `flow_lists`.

use ui::assets::IconName;
use ui::component::button::Button;
use ui::component::checkbox::Checkbox;
use ui::component::tag::Tag;
use ui::component::{Disableable as _, Icon, Size};
use ui::onboarding::kit::{self as grid};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, px, AnyElement, Context, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, SharedString, Styled as _,
};

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

/// A note: a grey card with the "i" beside its title and its words. The
/// library's alert pins its icon to the top at its own size, so the card
/// is laid out here, with the library's icon.
pub(super) fn note(
    id: &'static str,
    (title, words): (impl Into<SharedString>, impl Into<SharedString>),
) -> impl IntoElement {
    let (pad_x, pad_y) = fourth::CARD_PAD;
    div()
        .id(id)
        .w_full()
        .px(px(pad_x - 1.))
        .py(px(pad_y - 1.))
        .flex()
        .items_center()
        .gap_3()
        .rounded(px(fourth::CARD_CORNER))
        .border_1()
        .border_color(page::hairline())
        .bg(design::field())
        .child(
            Icon::from(IconName::Information)
                .size(px(design::NOTE_ICON))
                .flex_none()
                .text_color(page::ink()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    text(design::NOTE_TITLE)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(page::ink())
                        .child(title.into()),
                )
                .child(
                    text(design::NOTE_TEXT)
                        .text_color(fourth::faint())
                        .child(words.into()),
                ),
        )
}

/// A name with a line about it under it.
pub(super) fn named(name: impl Into<SharedString>, about: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            text(fourth::ROW_TITLE)
                .font_weight(FontWeight::MEDIUM)
                .text_color(page::ink())
                .child(name.into()),
        )
        .child(
            text(design::SMALL)
                .text_color(fourth::faint())
                .child(about.into()),
        )
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
    fn created(&self, cx: &mut Context<Self>) -> AnyElement {
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
                    .child(caption("ISONOMIC ID \u{b7} MADE ON THIS MAC"))
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
                                    .child(
                                        self.account
                                            .as_ref()
                                            .map_or_else(String::new, |account| account.id.clone()),
                                    ),
                            )
                            .child(held(
                                outlined(
                                    "onboarding-copy",
                                    if self.copied { "Copied" } else { "Copy" },
                                    None,
                                )
                                .on_click(cx.listener(|view, _, _, cx| view.copy_id(cx))),
                            )),
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
                    .child(caption("THIS ID IS THE WORKSPACE ROOT"))
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
                                .py_0p5()
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
        let cells = (1..=steps::WORD_COUNT)
            .map(|place| {
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
                            .child(format!("{place}.")),
                    )
                    .child(
                        text(fourth::ROW_TITLE)
                            .text_color(page::ink())
                            .child(self.word(place)),
                    )
                    .into_any_element()
            })
            .collect();
        let under = match (&self.recovery, self.kit_asked) {
            (Some(kit), _) => format!(
                "{} \u{b7} Keep the words and the kit file apart, away from this Mac",
                steps::short(&kit.id)
            ),
            (None, true) => {
                "Asking Isonomic\u{2019}s engine for your recovery kit\u{2026}".to_owned()
            }
            (None, false) => "Your recovery kit isn\u{2019}t here yet.".to_owned(),
        };
        let again = (self.recovery.is_none() && !self.kit_asked).then(|| {
            held(
                outlined("onboarding-kit-again", "Try again", None)
                    .on_click(cx.listener(|view, _, _, cx| view.fetch_kit(cx))),
            )
        });
        let (says, mark) = match self.kit_file {
            Some(_) => ("Kit file saved", IconName::Check),
            None => ("Save kit file", IconName::Download),
        };
        let save = outlined("onboarding-save-kit", says, Some(mark))
            .disabled(self.recovery.is_none())
            .on_click(cx.listener(|view, _, _, cx| view.save_kit(cx)));
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(caption("ISONOMIC RECOVERY KIT \u{b7} 24 WORDS"))
            .children(word_rows(cells))
            .child(text(design::SMALL).text_color(fourth::faint()).child(under))
            .child(
                div()
                    .w_full()
                    .py_1p5()
                    .flex()
                    .items_center()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Checkbox::new("onboarding-stored")
                            .checked(self.stored)
                            .label("I\u{2019}ve stored this somewhere safe, off this machine.")
                            .on_click(cx.listener(|view, stored: &bool, _, cx| {
                                view.store_kit(*stored, cx)
                            })),
                    )
                    .child(div().flex_1())
                    .children(again)
                    .child(held(save)),
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
                    .child(named(*name, *about).max_w(px(design::SETTING_WIDE)))
                    .child(div().flex_1())
                    .child(self.switch(
                        steps::FINISH,
                        index,
                        (
                            self.residency[index],
                            index == 0 && self.backend.starts_at_login().is_none(),
                        ),
                        cx,
                    ));
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
            .child(card().py_1p5().children(rows))
            .child(note("onboarding-finish-note", steps::AFTER))
            .into_any_element()
    }

    /// The body of step `index`'s panel.
    pub(super) fn body(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        match index {
            steps::CREATED => self.created(cx),
            steps::KIT => self.kit(cx),
            steps::CONFIRM => self.confirm(),
            steps::MODELS => self.models_list(cx),
            steps::LINKS => self.links_list(cx),
            _ => self.finish(cx),
        }
    }
}
