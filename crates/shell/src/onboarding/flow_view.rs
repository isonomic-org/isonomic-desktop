// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The later steps' drawing: each step's art in the sidebar, its panel,
//! the Face ID sheet, and "Setting up your workspace". The panels' bodies
//! are in `flow_bodies`.

use ui::assets::IconName;
use ui::component::spinner::Spinner;
use ui::component::{Icon, Sizable as _, Size};
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design, at};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    canvas, div, img, logo, motion, point, px, Context, FontWeight, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Styled as _, TestSupportExt as _,
};

use super::art::Sprite;
use super::flow::{self, Flow, Slip};
use super::flow_bodies::{held, outlined};
use super::steps::{self, STEPS};
use super::Onboarding;

/// How far below its place something starts so that it is under the
/// window's bottom edge.
fn below(top: f32) -> f32 {
    page::HEIGHT - top + 8.
}

/// A sprite moved `shift` to the right and drawn `narrow` of its width
/// about the art's middle line, as when the whole art turns over. The ID
/// card is also wherever `slip` has it.
fn sprite(sprite: &Sprite, shift: f32, narrow: f32, slip: Slip) -> impl IntoElement {
    let (left, top, wide, tall) = sprite.place;
    let (left, top, narrow) = if sprite.slips {
        (left + slip.across, top + slip.down, narrow * slip.narrow)
    } else {
        (left, top, narrow)
    };
    let middle = left + wide / 2.;
    let drawn = wide * narrow;
    img(&sprite.picture)
        .absolute()
        .left(px(design::ART_X + (middle - design::ART_X) * narrow
            - drawn / 2.
            + shift))
        .top(px(top))
        .w(px(drawn))
        .h(px(tall))
}

impl Onboarding {
    /// What the stepper says at `t`: its label, the step's name, what it
    /// is about, how many bars are filled, and how solid the words are
    /// while they change.
    pub(super) fn stepper_now(
        &self,
        t: f32,
    ) -> (&'static str, &'static str, &'static str, usize, f32) {
        let terms = (
            "STEP 1",
            "Accept terms",
            "Isonomic runs on your Mac and you hold your own keys.",
            1,
        );
        let of = |step: usize| {
            let step = &STEPS[step];
            (step.label, step.name, step.about, step.bars)
        };
        let account = of(0);
        // Out, then in.
        let pick = |change: f32, from: (_, _, _, usize), to: (_, _, _, usize)| {
            let (label, name, about, bars) = if change < 0.5 { from } else { to };
            (label, name, about, bars, (1. - 2. * change).abs())
        };
        match (self.flow_at(t), self.passkey_at(t)) {
            (Some((Flow { step, forward, .. }, f)), _) => {
                let from = step.checked_sub(1).map_or(account, of);
                let change = if forward {
                    motion::progress(f, 0., at::WORDS_CHANGE)
                } else {
                    1.
                };
                pick(change, from, of(step))
            }
            (None, Some(p)) => pick(motion::progress(p, 0., at::WORDS_CHANGE), terms, account),
            (None, None) => pick(0., terms, terms),
        }
    }

    /// How solid the dotted line is while a step's art changes: it fades
    /// with the old and comes back with the new.
    pub(super) fn dots_through_flow(&self, t: f32) -> f32 {
        match self.flow_at(t) {
            // The holder stays through "Confirm your recovery kit".
            Some((
                Flow {
                    step: steps::CONFIRM,
                    ..
                },
                _,
            )) => 1.,
            Some((Flow { forward: true, .. }, f)) => {
                1. - motion::progress(f, 0., at::DOTS_FADE)
                    + motion::progress(f, at::ART_ARRIVES + at::FLIGHT - 0.1, at::DOTS_FADE)
            }
            _ => 1.,
        }
    }

    /// A step's art: leaving by the left edge, arriving from under the
    /// panel, or at rest. The ID card arrives after the wallet, and lies
    /// over it until it is tucked in.
    fn art(&self, step: usize, leaving: bool, f: f32, slip: Slip) -> impl IntoElement {
        let sprites = self.art.as_ref().map(|art| &art.steps[step]);
        let leftmost = sprites
            .into_iter()
            .flatten()
            .map(|sprite| sprite.place.0)
            .fold(f32::MAX, f32::min);
        let placed = |f: f32| {
            if leaving {
                let gone = flow::left(f);
                (-(shared::SIDEBAR + 40.) * gone, flow::turned(gone))
            } else {
                let arrived = flow::arrived(f);
                (
                    (shared::SIDEBAR + shared::TUCK - leftmost) * (1. - arrived),
                    flow::turned(1. - arrived),
                )
            }
        };
        let (whole, card) = (placed(f), placed(f - at::CARD_FOLLOWS));
        let (under, over): (Vec<_>, Vec<_>) = sprites
            .into_iter()
            .flatten()
            .partition(|sprite| slip.tucked || !sprite.slips);
        div()
            .absolute()
            .size_full()
            .children(under.into_iter().chain(over).map(|one| {
                let (shift, narrow) = if one.slips && !leaving { card } else { whole };
                sprite(one, shift, narrow, slip)
            }))
    }

    /// A step's panel, `slid` of the way in from the right.
    fn step_panel(&self, index: usize, slid: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let step = &STEPS[index];
        let (heading, heading_line) = shared::HEADING;
        let (sub, sub_line) = shared::SUBHEADING;
        let (header_top, header_bottom) = shared::HEADER_PAD;
        let away = page::WIDTH - shared::SIDEBAR + 8.;
        // A list step has a button beside its heading and its tabs under
        // it, all above the rule.
        let (add, filters) = self.header_extras(index, cx).unzip();
        let listed = filters.is_some();
        let titles = div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(if listed { 6. } else { 10. }))
            .child(
                div()
                    .text_size(px(heading))
                    .line_height(px(heading_line))
                    .font_weight(FontWeight::BOLD)
                    .text_color(page::ink())
                    .child(step.heading),
            )
            .child(
                div()
                    .max_w(px(if step.ruled {
                        fourth::CONTENT.2
                    } else {
                        fourth::HEAD
                    }))
                    .text_size(px(sub))
                    .line_height(px(sub_line))
                    .text_color(shared::quiet())
                    .child(step.lead),
            );
        let header = div()
            .flex_none()
            .w_full()
            .pt(px(header_top))
            .pb(px(header_bottom + if listed { 2. } else { 0. }))
            .px(px(shared::PAD))
            .flex()
            .flex_col()
            .gap_6()
            .border_color(page::hairline())
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_start()
                    .gap_6()
                    .child(titles)
                    .children(add),
            )
            .children(filters);
        let header = if step.ruled {
            header.border_b_1()
        } else {
            header.pb_0()
        };
        div()
            .id(step.id)
            .test_support()
            .absolute()
            .left(px(shared::SIDEBAR + away * (1. - slid)))
            .top_0()
            .w(px(page::WIDTH - shared::SIDEBAR))
            .h_full()
            // Opaque: it covers the last panel, and the sidebar's art
            // starts hidden under it.
            .rounded_tr(page::corner())
            .rounded_br(page::corner())
            .bg(page::page())
            .flex()
            .flex_col()
            .child(header)
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .px(px(shared::PAD))
                    .pt(px(if listed { 0. } else { 20. }))
                    .child(self.body(index, cx)),
            )
            .child(
                div()
                    .flex_none()
                    .w_full()
                    .h(px(shared::FOOTER))
                    .px(px(shared::FOOTER_PAD))
                    .border_t_1()
                    .border_color(page::hairline())
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(self.action(
                        "onboarding-back",
                        "Previous step",
                        false,
                        Onboarding::back_step,
                        cx,
                    ))
                    .child(self.action(
                        "onboarding-next",
                        step.forward,
                        true,
                        Onboarding::next_step,
                        cx,
                    )),
            )
    }

    /// The Face ID sheet, as the system's would come up, `solid` of the
    /// way in.
    fn sheet(&self, solid: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let (wide, tall) = design::SHEET;
        div()
            .id("onboarding-face-id-sheet")
            .test_support()
            .absolute()
            .size_full()
            .rounded(page::corner())
            .bg(design::dim().opacity(design::SHEET_DIM * solid))
            // Nothing under the sheet can be pressed.
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .absolute()
                    .left(px((page::WIDTH - wide) / 2.))
                    .top(px((page::HEIGHT - tall) / 2.))
                    .w(px(wide))
                    .rounded(px(design::SHEET_CORNER))
                    .bg(page::page())
                    .opacity(solid)
                    .h(px(tall))
                    .px(px(design::SHEET_PAD.0))
                    .pt(px(design::SHEET_PAD.0))
                    .pb(px(design::SHEET_PAD.1))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(design::SHEET_GAP))
                    .child(
                        Icon::from(IconName::FingerprintRecognition)
                            .size(px(design::SHEET_ICON))
                            .text_color(page::ink()),
                    )
                    .child(
                        div()
                            .text_size(px(design::SHEET_TITLE.0))
                            .line_height(px(design::SHEET_TITLE.1))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(page::ink())
                            .child("Isonomic"),
                    )
                    .child(
                        div()
                            .text_size(px(design::SHEET_TEXT.0))
                            .line_height(px(design::SHEET_TEXT.1))
                            .text_color(shared::body())
                            .child("Create a passkey for \u{201c}isonomic.com\u{201d}"),
                    )
                    .child(Spinner::new().with_size(Size::Size(px(design::SHEET_SPINNER))))
                    .child(
                        div()
                            .font_family(shared::SERIF)
                            .text_size(px(design::SHEET_STATUS.0))
                            .line_height(px(design::SHEET_STATUS.1))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(page::ink())
                            .child("Looking for your face\u{2026}"),
                    )
                    .child(held(
                        outlined("onboarding-cancel", "Cancel", None)
                            .on_click(cx.listener(|view, _, _, cx| view.cancel_face_id(cx))),
                    )),
            )
    }

    /// "Setting up your workspace", `g` seconds in: everything clears, the
    /// laptop goes to the middle, and its name and the pill come up.
    fn setting_up(&self, g: f32) -> impl IntoElement {
        let cleared = motion::progress(g, 0., at::CLEAR);
        let (from_left, top, wide, tall) = shared::LAPTOP;
        let margin = (page::LAPTOP_CANVAS - 1.) / 2.;
        let laptop = self.frame.as_ref().map(|frame| {
            // Placed by the time its frame was drawn for; only across.
            let drawn_for = self.finishing.map_or(g, |began| frame.at - began);
            let (there, _) = flow::centring(drawn_for.max(0.));
            let across = motion::between(from_left, design::SETUP_LAPTOP, there);
            img(frame.image.clone())
                .absolute()
                .left(px(across - wide * margin))
                .top(px(top - tall * margin))
                .w(px(wide * page::LAPTOP_CANVAS))
                .h(px(tall * page::LAPTOP_CANVAS))
        });
        let mark = canvas(
            |_, _, _| (),
            |bounds, _, window, _| {
                let (x, y) = page::MARK_REST_CENTRE;
                logo::paint_at(
                    point(bounds.origin.x + px(x), bounds.origin.y + px(y)),
                    px(page::MARK_REST),
                    0.,
                    logo::LAST_FRAME,
                    page::ink(),
                    window,
                );
            },
        )
        .absolute()
        .size_full();
        let rise =
            |starts: f32| motion::ease(motion::ARRIVE, motion::progress(g, starts, at::RISE));
        let (name_left, name_top, name_wide, name_tall) = design::SETUP_NAME;
        let (pill_left, pill_top, pill_wide, pill_tall) = design::SETUP_PILL;
        let name_risen = rise(at::PILL_RISES);
        let pill_risen = rise(at::PILL_RISES + 0.15);
        div()
            .id("onboarding-setting-up")
            .test_support()
            .absolute()
            .size_full()
            .occlude()
            .child(
                div()
                    .absolute()
                    .size_full()
                    .rounded(page::corner())
                    .bg(page::page())
                    .opacity(cleared),
            )
            .child(mark)
            .children(laptop)
            .child(
                div()
                    .absolute()
                    .left(px(name_left))
                    .top(px(name_top + below(name_top) * (1. - name_risen)))
                    .w(px(name_wide))
                    .h(px(name_tall))
                    .flex()
                    .justify_center()
                    .text_size(px(page::TITLE_SIZE))
                    .line_height(px(name_tall))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(page::ink())
                    .child(self.device.clone()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(pill_left))
                    .top(px(pill_top + below(pill_top) * (1. - pill_risen)))
                    .w(px(pill_wide))
                    .h(px(pill_tall))
                    .px_1p5()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .rounded(px(pill_tall / 2.))
                    .border_1()
                    .border_color(page::hairline())
                    .bg(design::field())
                    .shadow(page::button_glow())
                    .text_size(px(page::BUTTON_LABEL))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(page::ink())
                    .child(
                        div()
                            .flex_none()
                            .size(px(design::SETUP_MARK.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .border_1()
                            .border_color(design::accent().opacity(design::SETUP_RING))
                            .child(
                                div()
                                    .size(px(design::SETUP_MARK.1))
                                    .rounded_full()
                                    .border(px(page::RING_LINE))
                                    .border_color(design::accent()),
                            ),
                    )
                    .child(div().flex_1().child("Setting up your workspace"))
                    .child(
                        Spinner::new()
                            .color(design::accent())
                            .with_size(Size::Size(px(design::SETUP_MARK.0))),
                    ),
            )
    }

    /// Everything of the later steps, over the fourth screen.
    pub(super) fn flow_layer(&self, t: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let mut layer = div().absolute().size_full();
        if let Some((Flow { step, forward, .. }, f)) = self.flow_at(t) {
            let moving = forward && f < at::END;
            // The recovery kit's holder, under every panel.
            layer = layer.children(self.kit_layer(t));
            // The step before, while this one comes on over it.
            if let (true, Some(before)) = (moving, step.checked_sub(1)) {
                if f < at::LEAVE {
                    // The card slips into the wallet before they go.
                    layer = layer.child(self.art(before, true, f, self.slipped(t)));
                }
                if f < at::SLIDE {
                    layer = layer.child(self.step_panel(before, 1., cx));
                }
            }
            let f = if forward {
                f
            } else {
                at::END + at::CARD_FOLLOWS
            };
            let slid = motion::ease(motion::ARRIVE, motion::progress(f, 0., at::SLIDE));
            // Its art waits under the panel until the panel is in.
            if f >= at::ART_ARRIVES {
                layer = layer.child(self.art(step, false, f, Slip::RESTING));
            }
            layer = layer.child(self.step_panel(step, slid, cx));
        }
        let asked = self
            .asking
            .map(|asked| motion::progress(t - asked, 0., at::SHEET_FADE));
        let answered = self
            .answered
            .map(|answered| 1. - motion::progress(t - answered, 0., at::SHEET_FADE));
        if let Some(solid) = asked.or(answered).filter(|solid| *solid > 0.) {
            layer = layer.child(self.sheet(solid, cx));
        }
        if let Some(finishing) = self.finishing {
            layer = layer.child(self.setting_up(t - finishing));
        }
        layer
    }
}
