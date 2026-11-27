// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recovery kit's drawing: the holder and the padlock in the sidebar,
//! and the grid of words on "Confirm your recovery kit", three of them to
//! type back.

use ui::component::input::Input;
use ui::onboarding::kit::{self as design};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, FontWeight, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Styled as _, TestSupportExt as _,
};

use super::flow_bodies::{caption, note, text, word_rows};
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
            .filter(|_| self.lock_out(t))
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
    fn word_cell(&self, place: usize) -> AnyElement {
        let asked = steps::ASKED.iter().position(|asked| *asked == place);
        let number = text(fourth::ROW_TEXT).child(format!("{place}."));
        let typed = asked.zip(self.words.as_ref());
        let Some((index, words)) = typed else {
            return div()
                .flex_1()
                .h(px(design::CELL))
                .px(px(fourth::ROW_PAD))
                .flex()
                .items_center()
                .gap_2()
                .rounded(px(shared::ACTION_CORNER))
                .border_1()
                .border_color(page::hairline())
                .font_family(shared::SERIF)
                .opacity(design::SHOWN)
                .child(number.text_color(fourth::faint()))
                .child(
                    text(fourth::ROW_TITLE)
                        .text_color(fourth::faint())
                        .child(self.word(place)),
                )
                .into_any_element();
        };
        let input = Input::new(&words[index])
            .prefix(
                number
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(page::ink()),
            )
            .min_h(px(design::CELL));
        let input = if self.wrong[index] {
            // Wrong shows whether or not it has the keyboard.
            input.focus_bordered(false).border_color(design::wrong())
        } else {
            input
        };
        div()
            .id(("onboarding-word", index))
            .test_support()
            .flex_none()
            .w(px(design::CELL_WIDE))
            .font_family(shared::SERIF)
            // A press here is a press, not the start of moving the window.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(input)
            .into_any_element()
    }

    /// "Confirm your recovery kit": the kit's 24 words, three of them to
    /// type back.
    pub(super) fn confirm(&self) -> AnyElement {
        let cells = (1..=steps::WORD_COUNT)
            .map(|place| self.word_cell(place))
            .collect();
        let wrong = self.wrong.iter().position(|wrong| *wrong);
        let under = match (wrong, self.is_confirmed().then_some(())) {
            (Some(_), _) if self.words_typed() == 0 => text(fourth::ROW_TEXT)
                .text_color(design::wrong())
                .child("Type words 4, 11 and 19 from your kit to continue."),
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
        div()
            .id("onboarding-words")
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(caption(steps::CONFIRM_CAPTION))
            .children(word_rows(cells))
            .child(under)
            .child(div().h_1())
            .child(note("onboarding-confirm-note", steps::CONFIRM_NOTE))
            .into_any_element()
    }
}
