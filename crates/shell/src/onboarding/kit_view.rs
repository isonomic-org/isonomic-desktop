// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recovery kit's drawing: the holder and the padlock in the sidebar,
//! and the grid of words on "Confirm your recovery kit", three of them to
//! type back.

use ui::onboarding::kit::{self as design};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps as steps_design;
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    TestSupportExt as _,
};

use super::flow_bodies::{card, text};
use super::kit_run;
use super::steps;
use super::Onboarding;

impl Onboarding {
    /// The holder and, once it is locked, the padlock, at `t`. Each is
    /// placed by the time its frame was drawn for.
    pub(super) fn kit_layer(&self, t: f32) -> Option<impl IntoElement> {
        let (frame, _) = self.kit_frame.as_ref()?;
        let (wide, tall) = design::CANVAS;
        let (middle, down) = design::CENTRE;
        let (carried, narrow) = self.kit_carried(frame.at);
        let shaken = self.shaken.map_or(0., |shaken| kit_run::shake(t - shaken));
        let holder = img(frame.image.clone())
            .absolute()
            .left(px(middle - wide * narrow / 2. + carried + shaken))
            .top(px(down - tall / 2.))
            .w(px(wide * narrow))
            .h(px(tall));
        let lock = self
            .lock_frame
            .as_ref()
            .zip(self.kit.as_ref())
            .map(|((frame, _), kit)| {
                let (wide, tall) = design::LOCK_CANVAS;
                let (across, below) = kit.lock_place();
                img(frame.image.clone())
                    .absolute()
                    .left(px(middle + across * narrow - wide * narrow / 2.
                        + carried
                        + self.lock_coming(t)))
                    .top(px(down + below - tall / 2.))
                    .w(px(wide * narrow))
                    .h(px(tall))
            });
        Some(div().absolute().size_full().child(holder).children(lock))
    }

    /// One cell of the grid: a word the kit shows, or one to type.
    fn word_cell(&self, place: usize, cx: &mut Context<Self>) -> AnyElement {
        let asked = steps::ASKED.iter().position(|asked| *asked == place);
        let cell = div()
            .flex_1()
            .h(px(design::CELL))
            .px(px(fourth::ROW_PAD))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(shared::ACTION_CORNER))
            .border_1()
            .font_family(shared::SERIF);
        let number = text(fourth::ROW_TEXT).child(format!("{place}."));
        let Some(index) = asked else {
            return cell
                .border_color(page::hairline())
                .child(number.text_color(fourth::faint()))
                .child(
                    text(fourth::ROW_TITLE)
                        .text_color(fourth::faint())
                        .child(steps::WORDS[place - 1]),
                )
                .into_any_element();
        };
        let typing = self.confirmed.is_none() && self.typing == index;
        let edge = if self.wrong[index] {
            design::wrong()
        } else if typing {
            design::focus()
        } else {
            shared::outline()
        };
        // A bar where the next letter goes.
        let caret = typing.then(|| div().w(px(1.)).h(px(16.)).bg(page::ink()));
        cell.id(("onboarding-word", index))
            .test_support()
            .border_color(edge)
            .bg(page::page())
            .cursor_text()
            .on_click(cx.listener(move |view, _, window, cx| {
                view.words_focus.focus(window, cx);
                view.type_word(index, cx);
            }))
            .child(
                number
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(page::ink()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(
                        text(fourth::ROW_TITLE)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(page::ink())
                            .child(self.typed[index].clone()),
                    )
                    .children(caret),
            )
            .into_any_element()
    }

    /// "Confirm your recovery kit": the kit's 24 words, three of them to
    /// type back.
    pub(super) fn confirm(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = Vec::new();
        for row in 0..steps::WORDS.len() / design::ACROSS {
            let mut cells = Vec::new();
            for column in 0..design::ACROSS {
                cells.push(self.word_cell(row * design::ACROSS + column + 1, cx));
            }
            rows.push(
                div()
                    .w_full()
                    .flex()
                    .gap(px(design::CELL_GAP))
                    .children(cells),
            );
        }
        let wrong = self.wrong.iter().position(|wrong| *wrong);
        let under = match (wrong, self.confirmed) {
            (Some(index), _) => text(fourth::ROW_TEXT)
                .text_color(design::wrong())
                .child(format!(
                    "That doesn\u{2019}t match word {} in your kit. Check the spelling.",
                    steps::ASKED[index]
                )),
            (None, Some(_)) => text(fourth::ROW_TEXT)
                .text_color(shared::quiet())
                .child("3 of 3 words confirmed"),
            (None, None) => text(fourth::ROW_TEXT)
                .text_color(shared::quiet())
                .child(format!("{} of 3 words entered", self.words_typed())),
        };
        let (title, words) = steps::CONFIRM_NOTE;
        div()
            .id("onboarding-words")
            .track_focus(&self.words_focus)
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, _, cx| view.word_key(event, cx)))
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                text(fourth::CAPTION)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(fourth::faint())
                    .child(steps::CONFIRM_CAPTION),
            )
            .children(rows)
            .child(under)
            .child(div().h(px(steps_design::SMALL.0 - 8.)))
            .child(
                card()
                    .gap(px(2.))
                    .child(
                        text(fourth::ROW_TEXT)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(page::ink())
                            .child(title),
                    )
                    .child(
                        text(fourth::ROW_TEXT)
                            .text_color(shared::quiet())
                            .child(words),
                    ),
            )
            .into_any_element()
    }
}
