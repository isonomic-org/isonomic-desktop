// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The fourth screen's drawing: the seal and its rings of light in the
//! sidebar, and over them the panel that slid in, each piece placed where
//! the design places it.

use ui::onboarding::passkey::{self as design, at};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    div, img, motion, px, Context, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Styled as _, TestSupportExt as _,
};

use super::flow_bodies::{held, outlined};
use super::flow_view::TUCKED;
use super::stage::Drawn;
use super::{flow, passkey, Onboarding};

const TITLE: &str = "Create your account with Face ID";
const LEAD: &str = "Your account is a passkey. The private key is generated in this Mac's secure enclave and never leaves it \u{2014} Isonomic never sees it, and there is no seed phrase to lose.";
/// What happens next, in order: each step's name and what it means.
const STEPS: [(&str, &str); 3] = [
    ("Face ID prompt", "macOS asks for your face or fingerprint. This is the OS, not Isonomic."),
    ("Passkey created", "Stored in the secure enclave and synced through iCloud Keychain to your other devices."),
    ("Account address derived", "An ERC-4337 smart account is derived from the passkey. It deploys on your first transaction, gas sponsored."),
];

impl Onboarding {
    /// The seal, with its light, at `p` seconds into the fourth screen. It
    /// goes under the panel, which it comes out from.
    pub(super) fn seal_layer(&self, p: f32) -> impl IntoElement {
        let (left, top, side) = design::TILE;
        let (middle_x, middle_y) = (left + side / 2., top + side / 2.);
        // On the next step it turns over and leaves by the left edge.
        let gone = self
            .flow_at(self.passkey.map_or(0., |started| started + p))
            .map_or(0., |(_, f)| flow::left(f));
        let narrow = flow::turned(gone);
        let rings = (0..design::RINGS.len()).rev().map(move |index| {
            let (share, fill, line) = passkey::ring(p, index);
            let (fill, line) = (fill * (1. - gone), line * (1. - gone));
            let wide = design::RINGS[index].0 * share;
            div()
                .absolute()
                .left(px(middle_x - wide / 2.))
                .top(px(middle_y - wide / 2.))
                .size(px(wide))
                .rounded(px(wide / 2.))
                .border(px(design::RING_LINE))
                .border_color(design::light().opacity(line))
                .bg(design::light().opacity(fill))
        });
        let canvas = design::SEAL_FRAME * design::SEAL_CANVAS;
        // Under its panel it isn't drawn: going back, the panel leaves.
        let out = |frame: &&Drawn| {
            let drawn_for = self.passkey_at(frame.at).unwrap_or(p);
            passkey::flight(drawn_for).0 > TUCKED
        };
        let seal = self.seal_frame.as_ref().filter(out).map(|frame| {
            // Placed by the time its frame was drawn for.
            let drawn_for = self.passkey_at(frame.at).unwrap_or(p);
            let (arrived, _) = passkey::flight(drawn_for);
            let from = shared::SIDEBAR + shared::TUCK - (canvas - side) / 2.;
            img(frame.image.clone())
                .absolute()
                .left(px(motion::between(from, middle_x - canvas / 2., arrived)
                    + canvas * (1. - narrow) / 2.
                    - (shared::SIDEBAR + 40.) * gone))
                .top(px(middle_y - canvas / 2.))
                .w(px(canvas * narrow))
                .h(px(canvas))
        });
        div().absolute().size_full().children(rings).children(seal)
    }

    /// One step of what happens next.
    fn step(index: usize) -> impl IntoElement {
        let (name, meaning) = STEPS[index];
        let (number, number_line) = design::NUMBER;
        let (pad_x, pad_y) = design::NUMBER_PAD;
        let (title, title_line) = design::ROW_TITLE;
        let (text, text_line) = design::ROW_TEXT;
        let row = div()
            .w_full()
            .flex()
            .items_start()
            .gap(px(design::ROW_GAP))
            .py(px(design::ROW_PAD))
            .border_color(page::hairline());
        let row = if index == 0 { row } else { row.border_t_1() };
        row.child(
            div()
                .flex_none()
                .px(px(pad_x))
                .py(px(pad_y))
                .rounded(px(number_line))
                .bg(design::chip())
                .text_size(px(number))
                .line_height(px(number_line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(shared::quiet())
                .child((index + 1).to_string()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_size(px(title))
                        .line_height(px(title_line))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(page::ink())
                        .child(name),
                )
                .child(
                    div()
                        .w(px(design::ROW_WIDE))
                        .text_size(px(text))
                        .line_height(px(text_line))
                        .text_color(design::faint())
                        .child(meaning),
                ),
        )
    }

    /// The panel, which slides over the last one from the right.
    pub(super) fn passkey_panel(&self, p: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let slid = motion::ease(motion::ARRIVE, motion::progress(p, 0., at::SLIDE));
        let (left, top, wide) = design::CONTENT;
        let (heading, heading_line) = shared::HEADING;
        let (sub, sub_line) = shared::SUBHEADING;
        let (caption, caption_line) = design::CAPTION;
        let (card_x, card_y) = design::CARD_PAD;
        let away = page::WIDTH - shared::SIDEBAR + 8.;
        div()
            .id("onboarding-passkey")
            .test_support()
            .absolute()
            .left(px(shared::SIDEBAR + away * (1. - slid)))
            .top_0()
            .w(px(page::WIDTH - shared::SIDEBAR))
            .h_full()
            // Opaque: it covers the last panel, and the sidebar's props
            // start hidden under it.
            .rounded_tr(page::corner())
            .rounded_br(page::corner())
            .bg(page::page())
            .child(
                div()
                    .absolute()
                    .left(px(left))
                    .top(px(top))
                    .w(px(wide))
                    .flex()
                    .flex_col()
                    .gap(px(design::GAP))
                    .child(
                        div()
                            .w(px(design::HEAD))
                            .flex()
                            .flex_col()
                            .gap(px(design::HEAD_GAP))
                            .child(
                                div()
                                    .text_size(px(heading))
                                    .line_height(px(heading_line))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(page::ink())
                                    .child(TITLE),
                            )
                            .child(
                                div()
                                    .text_size(px(sub))
                                    .line_height(px(sub_line))
                                    .text_color(shared::quiet())
                                    .child(LEAD),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .px(px(card_x))
                            .py(px(card_y))
                            .rounded(px(design::CARD_CORNER))
                            .border_1()
                            .border_color(page::hairline())
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(caption))
                                    .line_height(px(caption_line))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(design::faint())
                                    .child("WHAT HAPPENS WHEN YOU CONTINUE"),
                            )
                            .children((0..STEPS.len()).map(Self::step)),
                    )
                    .child(div().flex().pt_1().child(held(outlined(
                        "onboarding-why",
                        "Why no password?",
                        None,
                    )))),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .bottom_0()
                    .w_full()
                    .h(px(shared::FOOTER))
                    .px(px(shared::FOOTER_PAD))
                    .border_t_1()
                    .border_color(page::hairline())
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(self.action(
                        "onboarding-previous",
                        "Previous step",
                        false,
                        Onboarding::previous_step,
                        cx,
                    ))
                    .child(self.action(
                        "onboarding-face-id",
                        "Continue with Face ID",
                        true,
                        Onboarding::continue_with_face_id,
                        cx,
                    )),
            )
    }
}
