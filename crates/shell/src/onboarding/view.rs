// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The onboarding window's drawing: every piece placed where the design
//! places it in the 1280 × 800 page, moved by the one clock.

use ui::component::Icon;
use ui::onboarding::terms::at as third;
use ui::onboarding::{self as design, at};
use ui::{
    canvas, div, img, logo, motion, point, px, Context, FontWeight, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

use super::{laptop, terms, Chosen, Onboarding};

/// How far below its place something starts so that it is under the
/// window's bottom edge.
fn below(top: f32) -> f32 {
    design::HEIGHT - top + 8.
}

/// How far something that is leaving has gone, `t` seconds after the
/// press on "Start new account"; nothing before it.
fn departed(t: Option<f32>) -> f32 {
    t.map_or(0., |t| {
        motion::ease(motion::DEPART, motion::progress(t, 0., third::LEAVE))
    })
}

/// The same for "Start new account", which leaves a step later.
fn departed_after(t: Option<f32>) -> f32 {
    departed(t.map(|t| t - third::LEAVE_STAGGER))
}

/// One of the two buttons, in the order they come up.
#[derive(Clone, Copy)]
struct Choice {
    id: &'static str,
    label: &'static str,
    /// The arrow's icon in the interface library's catalog.
    arrow: &'static str,
    chosen: Chosen,
}

const CHOICES: [Choice; 2] = [
    Choice {
        id: "onboarding-new-account",
        label: "Start new account",
        arrow: "icons/arrow-right.svg",
        chosen: Chosen::NewAccount,
    },
    Choice {
        id: "onboarding-recover",
        label: "Recover your account",
        arrow: "icons/arrow-down.svg",
        chosen: Chosen::Recover,
    },
];

impl Onboarding {
    fn button(&self, index: usize, now: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let Choice {
            id,
            label,
            arrow,
            chosen: choice,
        } = CHOICES[index];
        let (wide, tall) = design::BUTTON;
        let (pad_left, pad_right) = design::BUTTON_PAD;
        let top = design::BUTTON_TOPS[index];
        let starts = at::BUTTONS_RISE + at::BUTTON_STAGGER * index as f32;
        let risen = motion::ease(motion::ARRIVE, motion::progress(now, starts, at::RISE));
        // When a new account is started, that button leaves to the right
        // and the other drops out of the window.
        let (across, down) = if choice == Chosen::NewAccount {
            let gone = departed_after(self.terms_now());
            ((design::WIDTH - design::BUTTON_LEFT + 8.) * gone, 0.)
        } else {
            (0., below(top) * departed(self.terms_now()))
        };
        div()
            .id(id)
            .test_support()
            .absolute()
            .left(px(design::BUTTON_LEFT + across))
            .top(px(top + below(top) * (1. - risen) + down))
            .w(px(wide))
            .h(px(tall))
            .pl(px(pad_left))
            .pr(px(pad_right))
            .flex()
            .items_center()
            .justify_between()
            .rounded(px(tall / 2.))
            .border_1()
            .border_color(design::hairline())
            .bg(design::page())
            .shadow(design::button_glow())
            .cursor_pointer()
            .text_size(px(design::BUTTON_LABEL))
            .font_weight(FontWeight::MEDIUM)
            .text_color(design::ink())
            // A press here is a press, not the start of moving the window.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |view, _, _, cx| view.choose(choice, cx)))
            .child(SharedString::from(label))
            .child(
                div()
                    .size(px(design::RING))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(design::RING / 2.))
                    .border(px(design::RING_LINE))
                    .border_color(design::hairline())
                    .child(
                        Icon::default()
                            .path(arrow)
                            .size(px(design::ARROW))
                            .text_color(design::ink()),
                    ),
            )
    }
}

impl Render for Onboarding {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.started.is_none() {
            self.start(cx);
        }
        self.scale = window.scale_factor();
        let now = self.now();
        let third_now = self.terms_now();
        // The third screen's dotted line never stops pulsing.
        if now < at::END || third_now.is_some() {
            window.request_animation_frame();
        }

        // The mark: its intro, then the turn into the corner.
        let frame = logo::frame_at(std::time::Duration::from_secs_f32(now));
        let gone = motion::ease(
            motion::CARRY,
            motion::progress(now, at::MARK_LEAVES, at::MARK_TRAVEL),
        );
        let mark = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let (rest_x, rest_y) = design::MARK_REST_CENTRE;
                let centre = point(
                    bounds.origin.x + px(motion::between(design::WIDTH / 2., rest_x, gone)),
                    bounds.origin.y + px(motion::between(design::HEIGHT / 2., rest_y, gone)),
                );
                logo::paint_at(
                    centre,
                    px(motion::between(design::MARK_INTRO, design::MARK_REST, gone)),
                    design::MARK_SPIN * gone,
                    frame,
                    design::ink(),
                    window,
                );
            },
        )
        .absolute()
        .size_full();

        // The laptop: the turning model on its larger canvas, then the
        // finished picture over it. It flies up on the second screen, and
        // across to the sidebar on the third.
        let margin = (design::LAPTOP_CANVAS - 1.) / 2.;
        let (place, drop, settled) = match third_now {
            None => {
                // A drawn frame is placed by the time it was drawn for.
                let drawn_for = self.frame.as_ref().map_or(now, |frame| frame.at);
                let (landed, _, _) = laptop::flight(drawn_for);
                let (_, top, _, tall) = design::LAPTOP;
                let flight_ends = at::LAPTOP_RISES + at::LAPTOP_FLIGHT;
                (
                    design::LAPTOP,
                    (below(top) + tall * margin) * (1. - landed),
                    motion::progress(now, flight_ends, at::SETTLE),
                )
            }
            Some(t) => {
                let drawn_for = self.frame.as_ref().map_or(t, |frame| frame.at);
                let (gone, _) = terms::laptop_move(drawn_for);
                let move_ends = third::LAPTOP_MOVES + third::LAPTOP_MOVE;
                // Until its first frame is drawn the picture stays put.
                let settled = if self.frame.is_none() {
                    1.
                } else {
                    motion::progress(t, move_ends, third::SETTLE)
                };
                (terms::laptop_box(gone), 0., settled)
            }
        };
        let (left, top, wide, tall) = place;
        let turning = self.frame.as_ref().map(|frame| {
            img(frame.image.clone())
                .absolute()
                .left(px(left - wide * margin))
                .top(px(top - tall * margin + drop))
                .w(px(wide * design::LAPTOP_CANVAS))
                .h(px(tall * design::LAPTOP_CANVAS))
        });
        let picture = (settled > 0.).then(|| {
            img(self.picture.clone())
                .id("onboarding-laptop")
                .absolute()
                .left(px(left))
                .top(px(top))
                .w(px(wide))
                .h(px(tall))
                .opacity(settled)
        });

        let (title_left, title_top, title_wide, title_tall) = design::TITLE;
        let title_risen = motion::ease(
            motion::ARRIVE,
            motion::progress(now, at::TITLE_RISES, at::RISE),
        );
        let title = div()
            .id("onboarding-title")
            .test_support()
            .absolute()
            .left(px(title_left))
            .top(px(title_top
                + below(title_top) * (1. - title_risen)
                + below(title_top) * departed(third_now)))
            .w(px(title_wide))
            .h(px(title_tall))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(design::TITLE_SIZE))
            .line_height(px(title_tall))
            .font_weight(FontWeight::MEDIUM)
            .text_color(design::ink())
            .child("Welcome to Isonomic");

        let new_account = self.button(0, now, cx);
        let recover = self.button(1, now, cx);

        div()
            .id("onboarding")
            .test_support()
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(design::corner())
            .bg(design::page())
            // With no title bar, the page itself moves the window.
            .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
            .children(third_now.map(|t| self.terms_backdrop(t)))
            .child(mark)
            .children(turning)
            .children(picture)
            .child(title)
            .child(new_account)
            .child(recover)
            .children(third_now.map(|t| self.terms_front(t, cx)))
    }
}
