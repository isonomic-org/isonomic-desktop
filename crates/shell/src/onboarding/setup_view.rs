// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! "Setting up your workspace": everything clears, the laptop goes to the
//! middle, and its name and the pill come up. Under the pill is said what
//! is being fetched, with the way on without waiting for it.

use ui::component::link::Link;
use ui::component::spinner::Spinner;
use ui::component::{Sizable as _, Size};
use ui::onboarding::steps::{self as design, at};
use ui::onboarding::terms::{self as shared};
use ui::onboarding::{self as page};
use ui::{
    canvas, div, logo, motion, point, px, Context, FontWeight, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Styled as _, TestSupportExt as _,
};

use super::flow;
use super::Onboarding;

/// How far below its place something starts so that it is under the
/// window's bottom edge.
fn below(top: f32) -> f32 {
    page::HEIGHT - top + 8.
}

impl Onboarding {
    /// "Setting up your workspace", `g` seconds in: everything clears, the
    /// laptop goes to the middle, and its name and the pill come up.
    pub(super) fn setting_up(&self, g: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let cleared = motion::progress(g, 0., at::CLEAR);
        let (from_left, top, wide, tall) = shared::LAPTOP;
        let margin = (page::LAPTOP_CANVAS - 1.) / 2.;
        let laptop = self.frame.as_ref().map(|frame| {
            // Placed by the time its frame was drawn for; only across.
            let drawn_for = self.finishing.map_or(g, |began| frame.at - began);
            let (there, _) = flow::centring(drawn_for.max(0.));
            let across = motion::between(from_left, design::SETUP_LAPTOP, there);
            let canvas = (
                across - wide * margin,
                top - tall * margin,
                wide * page::LAPTOP_CANVAS,
                tall * page::LAPTOP_CANVAS,
            );
            super::laptop::shown(frame.image.clone(), canvas)
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
                    .child(div().max_w_full().truncate().child(self.device.clone())),
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
            .children(self.setup.as_ref().map(|(says, failed)| {
                // What is being fetched, under the pill, and the way on
                // without waiting for it.
                let (wide, gap) = design::SETUP_SAYS;
                div()
                    .absolute()
                    .left(px((page::WIDTH - wide) / 2.))
                    .top(px(pill_top + pill_tall + gap))
                    .w(px(wide))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .opacity(pill_risen)
                    .text_size(px(design::SMALL.0))
                    .line_height(px(design::SMALL.1))
                    .text_color(if *failed {
                        ui::onboarding::kit::wrong()
                    } else {
                        shared::quiet()
                    })
                    .child(div().max_w_full().text_center().child(says.clone()))
                    .children((!*failed && !self.set_up_done).then(|| {
                        div()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(
                                Link::new("onboarding-leave-downloading")
                                    .text_color(page::ink())
                                    .child("Continue while it downloads")
                                    .on_click(
                                        cx.listener(|view, _, _, cx| view.leave_it_downloading(cx)),
                                    ),
                            )
                    }))
            }))
    }
}
